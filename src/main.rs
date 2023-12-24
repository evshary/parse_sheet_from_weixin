mod error;
mod sheet;
mod video;

const OUTPUT: &str = "output";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let file_content = std::fs::read_to_string("urls.txt")?;
    let urls = file_content.split('\n');
    let mut failed_url = Vec::<&str>::new();
    for url in urls {
        // Add newline to separate
        log::info!("-----------------------------------------------------------------------------------------------");
        // Don't access the website too fast
        std::thread::sleep(std::time::Duration::new(5, 0));
        // Parse the resource
        let sheet = match sheet::Sheet::try_new(url.to_string()).await {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Failed to parse sheet: {:?}", e);
                failed_url.push(url);
                continue;
            }
        };
        // Download the resource
        match sheet.download(OUTPUT).await {
            Ok(_) => {}
            Err(e) => {
                log::warn!("Failed to download sheet: {:?}", e);
                failed_url.push(url);
                continue;
            }
        }
    }
    log::info!("-----------------------------------------------------------------------------------------------");
    if failed_url.len() != 0 {
        log::warn!("Something wrong! Failure urls: {:?}", failed_url);
    } else {
        log::info!("Complete successfully!");
    }
    Ok(())
}
