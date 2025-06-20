use std::io::Write;

use crate::errors;

pub struct Sheet {
    url: String,
    title: String,
    accompaniment: String,
    video: Option<String>,
    sheets: Vec<String>,
}

use thirtyfour::prelude::*;

impl Sheet {
    pub async fn try_new(url: String) -> anyhow::Result<Sheet> {
        log::info!("The URL: {}", url);

        // Use firefox to load the URL
        let caps = DesiredCapabilities::firefox();
        let driver = WebDriver::new("http://localhost:4444", caps).await?;
        match driver.goto(&url).await {
            Ok(()) => {}
            Err(e) => {
                log::info!("{:?}", e);
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
        let mut title = document
            .select(&selector)
            .nth(0) // Get first element
            .ok_or(errors::SheetError::GetFailed("sheet title".to_string()))?
            .inner_html();
        title.retain(|c| !"\t\r\n".contains(c));
        let splits = title.trim().split('|').collect::<Vec<&str>>();
        let title = String::from(splits[1]) + " - " + splits[0];
        log::info!("Parsed title: {}", title);

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
        log::info!("Parsed voice URL: {}", accompaniment);

        // Get the url of video
        let selector =
            scraper::Selector::parse("video").map_err(|_| errors::SheetError::ParseFailed)?;
        let video = document
            .select(&selector)
            .nth(0)
            .and_then(|e| e.value().attr("src"))
            .map(String::from);
        log::info!("Parsed video URL: {:?}", video);

        // Get the music sheet
        // Get the attr data-src of img with class js_insertlocalimg
        let selector =
            scraper::Selector::parse("img").map_err(|_| errors::SheetError::ParseFailed)?;
        let imgs = document.select(&selector).filter(|x| {
            x.value()
                .attr("class")
                .unwrap_or_default()
                .contains("js_insertlocalimg")
        });
        let sheets = imgs
            .map(|img| {
                if let Some(src) = img.value().attr("data-src") {
                    src.to_string()
                } else {
                    log::warn!("Unabel to get sheet url");
                    String::new()
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
            if let Some(video) = self.video.clone() {
                // Download video as a file
                let resp = reqwest::get(video).await?;
                let binary = resp.bytes().await?;
                let mut file = std::fs::File::create(format!("{path}/{}.mp4", self.title))?;
                file.write_all(&binary)?;
            } else {
                return Err(errors::SheetError::GetFailed("video url".to_string()).into());
            }
        }

        Ok(())
    }
}
