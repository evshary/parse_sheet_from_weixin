use std::{io::Write, sync::Mutex};

use async_trait::async_trait;
use thirtyfour::prelude::*;

use crate::{errors, video::Downloader};

#[allow(dead_code)]
const QQ_URLS_FILE: &str = "qq_urls.txt";

#[allow(dead_code)]
static VIDEO_IDX: Mutex<usize> = Mutex::new(0);

#[allow(dead_code)]
pub struct Downloader20240707;
impl Downloader20240707 {
    #[allow(dead_code)]
    fn get_video_stream_from_qq(html: &str) -> anyhow::Result<(String, String)> {
        let document = scraper::Html::parse_document(html);
        // Get the video title
        let selector =
            scraper::Selector::parse("title").map_err(|_| errors::SheetError::ParseFailed)?;
        let title = document
            .select(&selector)
            .nth(0) // Get first element
            .ok_or(errors::SheetError::GetFailed("video title".to_string()))?
            .inner_html();
        log::info!("Downloaded video title: {}", title);
        // Get video url
        let selector =
            scraper::Selector::parse("video").map_err(|_| errors::SheetError::ParseFailed)?;
        let video_url = document
            .select(&selector)
            .nth(0) // Get first element
            .ok_or(errors::SheetError::GetFailed("video url".to_string()))?
            .value()
            .attr("src")
            .ok_or(errors::SheetError::GetFailed("video url".to_string()))?;
        log::info!("Downloaded video url: {}", video_url);
        Ok((title, video_url.to_owned()))
    }
}

#[async_trait]
impl Downloader for Downloader20240707 {
    fn get_url(_document: &scraper::Html) -> anyhow::Result<String> {
        let file_content = std::fs::read_to_string(QQ_URLS_FILE)?;
        let urls = file_content.split('\n').collect::<Vec<_>>();

        let mut global_idx = VIDEO_IDX.lock().unwrap();
        let idx = *global_idx;
        *global_idx += 1;
        Ok(urls[idx].to_string())
    }

    async fn download_video(
        _title: String,
        url: String,
        path: String,
        timeout: u64,
    ) -> anyhow::Result<()> {
        // Send request via selenium
        let caps = DesiredCapabilities::chrome();
        let driver = WebDriver::new("http://localhost:9515", caps).await?;
        //driver
        //    .set_implicit_wait_timeout(Duration::new(timeout, 0))
        //    .await?;
        //let timeouts = driver.get_timeouts().await?;
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
        let (title, video_url) = Downloader20240707::get_video_stream_from_qq(&html)?;
        // Download video as a file
        let resp = reqwest::get(video_url).await?;
        let binary = resp.bytes().await?;
        let mut file = std::fs::File::create(format!("{path}/{title}.mp4"))?;
        file.write_all(&binary)?;
        driver.quit().await?;
        Ok(())
    }
}
