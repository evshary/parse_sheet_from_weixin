use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string("urls.txt")?;
    let urls = file_content.split('\n');
    for url in urls {
        println!("Parse URL: {}", url);
        let resp = reqwest::get(url).await?;
        let text = resp.text().await?;
        println!("{}", text);
    }
    Ok(())
}
