use std::io::Write;

use async_trait::async_trait;
use thirtyfour::prelude::*;

use crate::{errors, video::Downloader};

pub struct Downloader20231224;
impl Downloader20231224 {
    fn get_video_stream(html: &str) -> anyhow::Result<String> {
        let document = scraper::Html::parse_document(html);

        let selector =
            scraper::Selector::parse("video").map_err(|_| errors::SheetError::ParseFailed)?;
        let video_url = document
            .select(&selector)
            .nth(0)
            .ok_or(errors::SheetError::GetFailed("video url".to_string()))?
            .value()
            .attr("src")
            .ok_or(errors::SheetError::GetFailed("video url".to_string()))?;

        Ok(video_url.to_owned())
    }
}

#[async_trait]
impl Downloader for Downloader20231224 {
    fn get_url(_document: &scraper::Html) -> anyhow::Result<String> {
        Ok(String::new())
    }

    async fn download_video(
        title: String,
        url: String,
        path: String,
        timeout: u64,
    ) -> anyhow::Result<()> {
        // Send request via selenium
        let caps = DesiredCapabilities::chrome();
        let driver = WebDriver::new("http://localhost:9515", caps).await?;
        match driver.goto(url).await {
            Ok(()) => {}
            Err(e) => {
                log::info!("{:?}", e);
                log::info!("You can ignore this meesage.");
            }
        }
        // Waiting for selenium
        std::thread::sleep(std::time::Duration::new(timeout, 0));
        let html = driver.source().await?;
        let video_url = Downloader20231224::get_video_stream(&html)?;
        log::info!("Video stream url: {}", video_url);
        // Download video as a file
        let resp = reqwest::get(video_url).await?;
        let binary = resp.bytes().await?;
        let mut file = std::fs::File::create(format!("{path}/{title}.mp4"))?;
        file.write_all(&binary)?;
        driver.quit().await?;
        Ok(())
    }
}
