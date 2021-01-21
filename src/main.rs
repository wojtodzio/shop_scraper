use async_trait::async_trait;
use futures::FutureExt;
use reqwest::Client;
use select::{
    document::Document,
    predicate::{Class, Name, Predicate},
};
use std::{collections::HashMap, error::Error, vec::Vec};

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.141 Safari/537.36";

struct Page {
    title: &'static str,
    url: &'static str,
    last_value: usize,
}

#[async_trait]
trait Scraper {
    async fn scrape(&self, url: &'static str) -> Result<usize, Box<dyn Error>>;
}

struct Shop<'a> {
    title: &'static str,
    scraper: &'a dyn Scraper,
    pages: Vec<Page>,
}

impl Shop<'_> {
    async fn scrape_pages(self) {
        for page in self.pages.iter() {
            match self.scraper.scrape(page.url).await {
                Err(e) => println!("Error!"),
                Ok(new_value) => println!("old - new: {} - {}", page.last_value, new_value),
            }
        }
    }
}

struct CountStringScraper<'a> {
    client: &'a Client,
    count_string: &'static str,
}

#[async_trait]
impl Scraper for CountStringScraper<'_> {
    async fn scrape(&self, url: &'static str) -> Result<usize, Box<dyn Error>> {
        let resp = self.client.get(url).send().await?.text().await?;
        let count = resp.matches(self.count_string).count();
        Ok(count)
    }
}

async fn check_avans(client: &Client) -> Result<bool, Box<dyn Error>> {
    dbg!("avans started!");

    const URL: &str = "https://www.avans.pl/konsole-i-gry/playstation-5";
    const SUBCATEGORIES_COUNT_WHEN_UNAVAILABLE: usize = 7;

    let resp = client.get(URL).send().await?.text().await?;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::builder().user_agent(USER_AGENT).build()?;

    let media_expert = Shop {
        title: "Media Expert",
        scraper: &(CountStringScraper {
            client: &client,
            count_string: "niedostępny",
        }),
        pages: vec![
            (Page {
                title: "Drive",
                url: "https://www.mediaexpert.pl/gaming/playstation-5/konsole-ps5/konsola-sony-ps5",
                last_value: 1,
            }),
            (Page {
                title: "Digital",
                url: "https://www.mediaexpert.pl/gaming/playstation-5/konsole-ps5/konsola-sony-ps5-digital",
                last_value: 1,
            }),
            (Page {
                title: "Index",
                url: "https://www.mediaexpert.pl/gaming/playstation-5/konsole-ps5",
                last_value: 2,
            }),
        ],
    };
    let x_kom = Shop {
        title: "X-Kom",
        scraper: &(CountStringScraper {
            client: &client,
            count_string: "Wycofany",
        }),
        pages: vec![
            (Page {
                title: "Drive",
                url: "https://www.x-kom.pl/p/577878-konsola-playstation-sony-playstation-5.html",
                last_value: 3,
            }),
            (Page {
                title: "Digital",
                url: "https://www.x-kom.pl/p/592843-konsola-playstation-sony-playstation-5-digital.html",
                last_value: 3,
            }),
        ],
    };

    // media_expert.scrape_pages().await;
    // x_kom.scrape_pages().await;

    // let mut results = HashMap::new();

    // // tokio::join!(check_x_kom(&client), check_avans(&client));

    // results.insert("x-kom", check_x_kom(&client).boxed());
    // results.insert("awans", check_avans(&client).boxed());
    // results.insert("mediaexpert", check_mediaexpert(&client).boxed());

    // for (store_name, available) in results.iter_mut() {
    //     dbg!("in loop");
    //     println!("{}: {}", store_name, available.await?);
    // }

    Ok(())
}
