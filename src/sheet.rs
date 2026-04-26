use std::io::Write;

use crate::{
    errors,
    video::{Downloader, Downloader20251220},
};

pub struct Sheet {
    url: String,
    title: String,
    accompaniment: String,
    video: Option<String>,
    sheets: Vec<String>,
}

use thirtyfour::prelude::*;

impl Sheet {
    fn get_png_dimensions(binary: &[u8]) -> Option<(u32, u32)> {
        const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
        if binary.len() < 24 || binary[..8] != PNG_SIGNATURE {
            return None;
        }

        let width = u32::from_be_bytes(binary[16..20].try_into().ok()?);
        let height = u32::from_be_bytes(binary[20..24].try_into().ok()?);
        Some((width, height))
    }

    fn is_likely_sheet_png(binary: &[u8]) -> bool {
        Self::get_png_dimensions(binary)
            .map(|(width, height)| width >= 500 && height >= 500)
            .unwrap_or(false)
    }

    fn get_image_url(img: scraper::element_ref::ElementRef<'_>) -> Option<String> {
        ["data-src", "src"]
            .into_iter()
            .find_map(|attr| img.value().attr(attr))
            .filter(|src| !src.is_empty() && !src.starts_with("data:"))
            .map(std::string::ToString::to_string)
    }

    fn is_sheet_image(url: &str) -> bool {
        url.contains("mmbiz.qpic.cn")
            && url.contains("wx_fmt=png")
            && url.contains("from=appmsg")
            && url.contains("#imgIndex=")
    }

    pub async fn try_new(url: String, index: usize) -> anyhow::Result<Sheet> {
        log::info!("The URL: {url}");

        // Use firefox to load the URL
        let caps = DesiredCapabilities::firefox();
        let driver = WebDriver::new("http://localhost:4444", caps).await?;
        match driver.goto(&url).await {
            Ok(()) => {}
            Err(e) => {
                log::info!("{e:?}");
                log::info!("You can ignore this meesage.");
            }
        }
        // Waiting for selenium
        std::thread::sleep(std::time::Duration::new(10, 0));

        // Get the HTML
        let html = driver.source().await?;
        let document = scraper::Html::parse_document(&html);

        // Get the title
        // Get the inner_html under h1
        let selector =
            scraper::Selector::parse("h1").map_err(|_| errors::SheetError::ParseFailed)?;
        let title = document
            .select(&selector)
            .nth(0) // Get first element
            .ok_or(errors::SheetError::GetFailed("sheet title".to_string()))?
            .text()
            .collect::<Vec<_>>()
            .join(" ");
        let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
        let splits = title
            .split('|')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        let title = if splits.len() >= 2 {
            format!("{} - {}", splits[1], splits[0])
        } else {
            title
        };
        log::info!("Parsed title: {title}");

        // Get the accompaniment
        // Get the attr voice_encode_fileid of mpvoice
        let selector = scraper::Selector::parse("mp-common-mpaudio")
            .map_err(|_| errors::SheetError::ParseFailed)?;
        let voice_id = document
            .select(&selector)
            .nth(0)
            .ok_or(errors::SheetError::GetFailed(
                "accompaniment url".to_string(),
            ))?
            .value()
            .attr("voice_encode_fileid")
            .ok_or(errors::SheetError::GetFailed(
                "accompaniment url".to_string(),
            ))?;
        let accompaniment = format!("https://res.wx.qq.com/voice/getvoice?mediaid={voice_id}");
        log::info!("Parsed voice URL: {accompaniment}");

        // Get the url of video
        let video = Downloader20251220::get_url_by_index(index).ok();
        log::info!("Parsed video URL: {video:?}");

        // Get the music sheet
        // Weixin article images do not always use the same class/attribute combination.
        // Prefer images inside the article body, and fall back to any image-like nodes
        // that expose a network URL.
        let selector =
            scraper::Selector::parse("#js_content img, .rich_media_content img, img")
                .map_err(|_| errors::SheetError::ParseFailed)?;
        let mut seen = std::collections::HashSet::new();
        let sheets = document
            .select(&selector)
            .filter_map(Self::get_image_url)
            .filter(|src| Self::is_sheet_image(src))
            .filter(|src| seen.insert(src.clone()))
            .collect::<Vec<String>>();
        log::info!("Parsed sheet URL: {sheets:?}");
        Ok(Sheet {
            url,
            title,
            accompaniment,
            video,
            sheets,
        })
    }

    pub async fn download(&self, path: &str) -> anyhow::Result<()> {
        // Create folder
        log::info!("Creating folder...");
        let path = format!("{}/{}", path, self.title.as_str());
        std::fs::create_dir_all(&path)?;

        // Create README
        {
            log::info!("Creating README...");
            let mut file = std::fs::File::create(format!("{path}/README"))?;
            file.write_all(self.url.as_bytes())?;
        }

        // Download accompaniment
        {
            log::info!("Dowloading accompaniment...");
            let resp = reqwest::get(self.accompaniment.clone()).await?;
            let binary = resp.bytes().await?;
            let mut file = std::fs::File::create(format!("{path}/伴奏.mp3"))?;
            file.write_all(&binary)?;
        }

        // Download sheet
        {
            log::info!("Dowloading sheets...");
            let mut saved_idx = 1;
            for sheet in self.sheets.clone() {
                let resp = reqwest::get(sheet).await?;
                let binary = resp.bytes().await?;
                if !Self::is_likely_sheet_png(&binary) {
                    log::info!("Skipping non-sheet image candidate");
                    continue;
                }
                let mut file = std::fs::File::create(format!("{}/{}.png", path, saved_idx))?;
                file.write_all(&binary)?;
                saved_idx += 1;
            }
        }

        // Download video
        {
            log::info!("Dowloading video...");
            if let Some(video) = self.video.clone() {
                // Download video as a file
                Downloader20251220::download_video(self.title.clone(), video, path, 0).await?;
            } else {
                return Err(errors::SheetError::GetFailed("video url".to_string()).into());
            }
        }

        Ok(())
    }
}
