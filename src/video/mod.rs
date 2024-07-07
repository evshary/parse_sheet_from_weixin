pub mod downloader20230525;
pub mod downloader20231224;

use async_trait::async_trait;
#[allow(unused_imports)]
pub use downloader20230525::Downloader20230525;
pub use downloader20231224::Downloader20231224;

#[async_trait]
pub trait Downloader {
    fn get_url(html: &scraper::Html) -> anyhow::Result<String>;
    async fn download_video(
        title: String,
        url: String,
        path: String,
        timeout: u64,
    ) -> anyhow::Result<()>;
}
