use reqwest::{cookie::Jar, ClientBuilder, Url};
use scraper::{Html, Selector};
use std::sync::Arc;

#[derive(Clone, PartialEq, Default)]
pub struct KaktusMessage {
    pub header: String,
    pub content: String,
}

pub async fn get_kaktus_page() -> Result<String, reqwest::Error> {
    // the page does not load without this cookie
    let cookie = "COOKIE_SUPPORT=true;";
    let url = "https://www.mujkaktus.cz/homepage".parse::<Url>().unwrap();

    let cookie_store = Arc::new(Jar::default());
    cookie_store.add_cookie_str(cookie, &url);

    let client = ClientBuilder::new()
        .cookie_provider(cookie_store.clone())
        .build()?;

    let request = client.get(url).build()?;
    let response = client.execute(request).await?;

    response.text().await
}

pub fn get_latest_message(page: &str) -> Option<KaktusMessage> {
    let document = Html::parse_document(page);

    // unwraps are safe, since the selectors are valid
    let article_selector = Selector::parse("div.box-bubble div.journal-content-article").unwrap();
    let h3_selector = Selector::parse("h3").unwrap();
    let p_selector = Selector::parse("p").unwrap();

    let latest_message = document.select(&article_selector).next()?;

    let latest_header = latest_message.select(&h3_selector).next()?.text().next()?;
    let latest_text = latest_message.select(&p_selector).next()?.text().next()?;

    Some(KaktusMessage {
        header: latest_header.to_string(),
        content: latest_text.to_string(),
    })
}
