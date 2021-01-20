use futures::FutureExt;
use reqwest::Client;
use select::{
    document::Document,
    predicate::{Class, Name, Predicate},
};
use std::{collections::HashMap, error::Error};

async fn check_avans(client: &Client) -> Result<bool, Box<dyn Error>> {
    dbg!("avans started!");
    const AVANS_URL: &str = "https://www.avans.pl/konsole-i-gry/playstation-5";
    const SUBCATEGORIES_COUNT_WHEN_UNAVAILABLE: usize = 7;

    let resp = client.get(AVANS_URL).send().await?.text().await?;
    let document = Document::from(&resp[..]);
    let matching_subcategories = document.find(Class("v-product_categories").descendant(
        Class("is-main").descendant(Class("is-subCategories").descendant(Name("div").descendant(
            Name("div").descendant(
                Class("is-subCategories").descendant(Name("ul").descendant(Name("li"))),
            ),
        ))),
    ));
    let available = matching_subcategories.count() != SUBCATEGORIES_COUNT_WHEN_UNAVAILABLE;

    dbg!("avans finished!");
    Ok(available)
}

async fn check_x_kom(client: &Client) -> Result<bool, Box<dyn Error>> {
    dbg!("x_kom started!");
    const X_KOM_URL: &str =
        "https://www.x-kom.pl/p/577878-konsola-playstation-sony-playstation-5.html";
    let resp = client.get(X_KOM_URL).send().await?.text().await?;
    let available = !resp.contains("Wycofany");

    dbg!("x_kom finished!");
    Ok(available)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.141 Safari/537.36";
    let client = Client::builder().user_agent(USER_AGENT).build()?;

    let mut results = HashMap::new();

    // tokio::join!(check_x_kom(&client), check_avans(&client));

    results.insert("x-kom", check_x_kom(&client).boxed());
    results.insert("awans", check_avans(&client).boxed());

    for (store_name, available) in results.iter_mut() {
        dbg!("in loop");
        println!("{}: {}", store_name, available.await?);
    }

    Ok(())
}
