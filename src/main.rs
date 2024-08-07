use reqwest::{blocking::ClientBuilder, cookie::Jar, Url};
use scraper::{Html, Selector};
use std::fs::{read_to_string, write};
use std::sync::Arc;

const FILE_PATH: &str = "last_message";

fn get_kaktus_page() -> Result<String, reqwest::Error> {
    // the page does not load without this cookie
    let cookie = "COOKIE_SUPPORT=true;";
    let url = "https://www.mujkaktus.cz/homepage".parse::<Url>().unwrap();

    let cookie_store = Arc::new(Jar::default());
    cookie_store.add_cookie_str(cookie, &url);

    let client = ClientBuilder::new()
        .cookie_provider(cookie_store.clone())
        .build()?;

    let request = client.get(url).build()?;
    let response = client.execute(request)?;

    response.text()
}

fn get_latest_message(page: &str) -> String {
    let document = Html::parse_document(page);

    let article_selector = Selector::parse("div.box-bubble div.journal-content-article").unwrap();
    let latest_message = document.select(&article_selector).next().unwrap();

    // TODO proper error handling
    let latest_header = latest_message
        .select(&Selector::parse("h3").unwrap())
        .next()
        .unwrap()
        .text()
        .next()
        .unwrap();
    let latest_text = latest_message
        .select(&Selector::parse("p").unwrap())
        .next()
        .unwrap()
        .text()
        .next()
        .unwrap();

    format!("{latest_header};{latest_text}")
}

// TODO use cron to run the script every few hours

fn main() {
    let old_message = match read_to_string(FILE_PATH) {
        Ok(content) => content,
        Err(_) => String::new(),
    };

    let Ok(page) = get_kaktus_page() else {
        println!("Unable to download the page");
        return;
    };
    let new_message = get_latest_message(&page);

    // TODO when new message empty -> velky spatny

    if old_message != new_message {
        println!("AAAAA");
        write(FILE_PATH, new_message).unwrap();
    }
}
