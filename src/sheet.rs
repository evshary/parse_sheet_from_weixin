use crate::error;
use crate::video::*;
use std::io::Write;

pub struct Sheet {
    url: String,
    title: String,
    accompaniment: String,
    video: String,
    sheets: Vec<String>,
}

impl Sheet {
    pub async fn try_new(url: String) -> anyhow::Result<Sheet> {
        // Get Html
        log::info!("The URL: {}", url);
        let resp = reqwest::get(url.clone()).await?;
        let html = resp.text().await?;
        let document = scraper::Html::parse_document(&html);

        // Get the title
        // Get the inner_html under h1
        let selector =
            scraper::Selector::parse("h1").map_err(|_| error::SheetError::ParseFailed)?;
        let mut title = document
            .select(&selector)
            .nth(0) // Get first element
            .ok_or(error::SheetError::GetFailed("sheet title".to_string()))?
            .inner_html();
        title.retain(|c| !"\t\r\n".contains(c));
        let splits = title.trim().split('|').collect::<Vec<&str>>();
        let title = String::from(splits[1]) + " - " + splits[0];
        log::info!("Parsed title: {}", title);

        // Get the accompaniment
        // Get the attr voice_encode_fileid of mpvoice
        let selector = scraper::Selector::parse("mp-common-mpaudio")
            .map_err(|_| error::SheetError::ParseFailed)?;
        let voice_id = document
            .select(&selector)
            .nth(0)
            .ok_or(error::SheetError::GetFailed(
                "accompaniment url".to_string(),
            ))?
            .value()
            .attr("voice_encode_fileid")
            .ok_or(error::SheetError::GetFailed(
                "accompaniment url".to_string(),
            ))?;
        let accompaniment = format!("https://res.wx.qq.com/voice/getvoice?mediaid={}", voice_id);
        log::info!("Parsed voice URL: {}", accompaniment);

        // Get the url of video
        // Get the attr data-src of iframe
        let video = VideoDownloader20230525::get_url(&document)?;
        log::info!("Parsed QQ video URL: {}", video);

        // Get the music sheet
        // Get the attr data-src of img with class js_insertlocalimg
        let selector =
            scraper::Selector::parse("img").map_err(|_| error::SheetError::ParseFailed)?;
        let imgs = document.select(&selector).filter(|x| {
            x.value()
                .attr("class")
                .unwrap_or_default()
                .contains("js_insertlocalimg")
        });
        let sheets = imgs
            .map(|img| match img.value().attr("data-src") {
                Some(src) => src.to_string(),
                None => {
                    log::warn!("Unabel to get sheet url");
                    "".to_owned()
                }
            })
            .collect::<Vec<String>>();
        log::info!("Parsed sheet URL: {:?}", sheets);
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
            let mut file = std::fs::File::create(format!("{}/README", path))?;
            file.write_all(self.url.as_bytes())?;
        }

        // Download accompaniment
        {
            log::info!("Dowloading accompaniment...");
            let resp = reqwest::get(self.accompaniment.clone()).await?;
            let binary = resp.bytes().await?;
            let mut file = std::fs::File::create(format!("{}/伴奏.mp3", path))?;
            file.write_all(&binary)?;
        }

        // Download sheet
        {
            log::info!("Dowloading sheets...");
            for (idx, sheet) in self.sheets.clone().into_iter().enumerate() {
                let resp = reqwest::get(sheet).await?;
                let binary = resp.bytes().await?;
                let mut file = std::fs::File::create(format!("{}/{}.png", path, idx + 1))?;
                file.write_all(&binary)?;
            }
        }

        // Download video
        {
            log::info!("Dowloading video...");
            // Timeout means we need to wait for ad play and load the video we want
            VideoDownloader20230525::download_video(self.video.clone(), path.clone(), 30).await?;
        }

        Ok(())
    }
}
