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
        let document = Html::parse_document(&text);
        // Get the title
        let selector = Selector::parse("h1").unwrap();
        let mut title = document
            .select(&selector)
            .nth(0) // Get first element
            .unwrap()
            .inner_html();
        title.retain(|c| !"\t\r\n".contains(c));
        let splits = title.trim().split('|').collect::<Vec<&str>>();
        let title = String::from(splits[1]) + " - " + splits[0];
        println!("Title: {}", title);

        // Get the accompaniment
        let selector = Selector::parse("mpvoice").unwrap();
        let voice_id = document
            .select(&selector)
            .nth(0)
            .unwrap()
            .value()
            .attr("voice_encode_fileid")
            .unwrap();
        println!(
            "Voice URL: https://res.wx.qq.com/voice/getvoice?mediaid={}",
            voice_id
        );
        // Get the url of video
        // Get the music sheet
    }
    Ok(())
}
