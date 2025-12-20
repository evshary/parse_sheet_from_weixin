use std::{process::Command, sync::Mutex};

use async_trait::async_trait;

use crate::video::Downloader;

#[allow(dead_code)]
const URLS_FILE: &str = "bilibili_urls.txt";

#[allow(dead_code)]
static VIDEO_IDX: Mutex<usize> = Mutex::new(0);

#[allow(dead_code)]
pub struct Downloader20251220;

#[async_trait]
impl Downloader for Downloader20251220 {
    fn get_url(_document: &scraper::Html) -> anyhow::Result<String> {
        let file_content = std::fs::read_to_string(URLS_FILE)?;
        let urls = file_content.split('\n').collect::<Vec<_>>();

        let mut global_idx = VIDEO_IDX.lock().unwrap();
        let idx = *global_idx;
        *global_idx += 1;
        Ok(urls[idx].to_string())
    }

    async fn download_video(
        title: String,
        url: String,
        path: String,
        _timeout: u64,
    ) -> anyhow::Result<()> {
        let output = format!("{path}/{title}.%(ext)s");
        let status = Command::new("yt-dlp")
            .arg("-f")
            .arg("bestvideo+bestaudio")
            .arg("-o")
            .arg(output)
            .arg(url)
            .status()?;
        if !status.success() {
            return Err(anyhow::anyhow!("yt-dlp failed"));
        }
        Ok(())
    }
}
