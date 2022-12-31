use regex::Regex;
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
        // Get the inner_html under h1
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
        // Get the attr voice_encode_fileid of mpvoice
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
        // Get the attr data-src of iframe
        let selector = Selector::parse("iframe").unwrap();
        let qq_url = document
            .select(&selector)
            .nth(0)
            .unwrap()
            .value()
            .attr("data-src")
            .unwrap();
        let re = Regex::new(r"vid=([[:alnum:]]+)").unwrap();
        let vid = re.captures(qq_url).unwrap();
        println!("Video URL: https://v.qq.com/x/page/{}.html", &vid[1]);
        // Get the music sheet
        // Get the attr data-src of img with class js_insertlocalimg
    }
    Ok(())
}
