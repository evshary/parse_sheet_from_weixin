use scraper::{Html, Selector};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string("urls.txt")?;
    let urls = file_content.split('\n');
    for url in urls {
        println!("Parse URL: {}", url);
        let resp = reqwest::get(url).await?;
        let text = resp.text().await?;
        //println!("{}", text);
        let document = Html::parse_document(&text);
        let selector = Selector::parse("h1").unwrap();
        for title in document.select(&selector) {
            let mut title = title.inner_html();
            title.retain(|c| !"\t\r\n".contains(c));
            let splits = title.trim().split('|').collect::<Vec<&str>>();
            let title = String::from(splits[1]) + " - " + splits[0];
            println!("{}", title);
        }
    }
    Ok(())
}
