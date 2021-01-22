use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use header::COOKIE;
use reqwest::{header, Client};
use scraper::{Html, Selector};
use std::{error::Error, vec::Vec};

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.141 Safari/537.36";
const DEFAULT_COOKIE: &str = "ak_bmsc=7BFAD44CA8940D5EFDBD8F749D17FEA85C7BBD2492720000F0250960D6CE5C01~plySkN3dJhrJ56+R1TrjSoMwcibz8jWimZvfXGqkyLUuOekVMCjfVZyV4k/CudrSsspmUOC8IkZryZfAt5bLBZcfTZaWeEinGiJXDTCSIqDKOjp1Wp7Soh9W2PGevCuEeGC3af8TE5OL0Sy57JSlOG05r5KmJHawx88fGicxtzVy2rzpeyOahJ0/PyYsQ+8OXdsdvsp1XJjesmskaJ9VVydIXP1beAE3Y2tXg+a3qR8Fw=";

struct Page {
    title: &'static str,
    url: &'static str,
    last_value: usize,
}

#[async_trait]
trait Scraper: Sync {
    async fn scrape(&self, url: &'static str) -> Result<usize, Box<dyn Error>>;
}

struct Shop<'a> {
    title: &'static str,
    scraper: &'a dyn Scraper,
    pages: Vec<Page>,
}

impl Shop<'_> {
    async fn scrape_pages(&self) {
        for page in self.pages.iter() {
            match self.scraper.scrape(page.url).await {
                Err(e) => println!("Error! {}", e),
                Ok(new_value) => {
                    if page.last_value != new_value {
                        println!(
                            "A change detected! {} - {}: {}/{} ({})",
                            self.title, page.title, new_value, page.last_value, page.url
                        );
                    }
                }
            }
        }
    }
}

struct CountStringScraper<'a> {
    client: &'a Client,
    string_to_count: &'static str,
}

#[async_trait]
impl Scraper for CountStringScraper<'_> {
    async fn scrape(&self, url: &'static str) -> Result<usize, Box<dyn Error>> {
        let resp = self.client.get(url).send().await?.text().await?;
        let count = resp.matches(self.string_to_count).count();
        Ok(count)
    }
}

struct CountElementsScraper<'a> {
    client: &'a Client,
    element_selector: &'a Selector,
}

#[async_trait]
impl Scraper for CountElementsScraper<'_> {
    async fn scrape(&self, url: &'static str) -> Result<usize, Box<dyn Error>> {
        let resp = self.client.get(url).send().await?.text().await?;
        let document = Html::parse_document(&resp);
        let count = document.select(self.element_selector).count();
        Ok(count)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut headers = header::HeaderMap::new();
    headers.insert(COOKIE, DEFAULT_COOKIE.parse().unwrap());
    let client = Client::builder()
        .default_headers(headers)
        .user_agent(USER_AGENT)
        .build()?;

    let shops = [
        Shop {
            title: "Media Expert",
            scraper: &(CountStringScraper {
                client: &client,
                string_to_count: "niedostępny",
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
        },
        Shop {
            title: "X-Kom",
            scraper: &(CountStringScraper {
                client: &client,
                string_to_count: "Wycofany",
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
        },
        Shop {
            title: "Avans",
            scraper: &(CountElementsScraper {
                client: &client,
                element_selector: &Selector::parse("body > div > div.is-main > div.is-subCategories > div > div > div.is-subCategories > ul > li").unwrap(),
            }),
            pages: vec![
                (Page {
                    title: "Categories index",
                    url: "https://www.avans.pl/konsole-i-gry/playstation-5",
                    last_value: 7
                })
            ]
        },
        Shop {
            title: "Empik",
            scraper: &(CountStringScraper {
                client: &client,
                string_to_count: "Produkt niedostępny",
            }),
            pages: vec![
                (Page {
                    title: "Digital",
                    url: "https://www.empik.com/konsola-sony-playstation-5-digital-edition-sony,p1249986067,multimedia-p",
                    last_value: 1,
                }),
                (Page {
                    title: "Digital - preorder",
                    url: "https://www.empik.com/konsola-sony-playstation-5-digital-edition-preorder-sony-computer-entertainment-europe,p1249990459,multimedia-p",
                    last_value: 1,
                }),
                (Page {
                    title: "Drive - preorder",
                    url: "https://www.empik.com/konsola-sony-playstation-5-1-tb-sony,p1244094954,multimedia-p",
                    last_value: 1,
                }),
            ],
        },
        Shop {
            title: "Empik - index page",
            scraper: &(CountStringScraper {
                client: &client,
                string_to_count: "Nakład konsol został wyczerpany",
            }),
            pages: vec![
                (Page {
                    title: "Index",
                    url: "https://www.empik.com/gry-i-programy/playstation-5",
                    last_value: 1,
                }),
            ],
        },
        Shop {
            title: "Ultima",
            scraper: &(CountStringScraper {
                client: &client,
                string_to_count: "Nie znaleźliśmy żadnych produktów spełniających podane przez Ciebie kryteria wyszukiwania.",
            }),
            pages: vec![
                (Page {
                    title: "Index",
                    url: "https://www.ultima.pl/ct/playstation-5/sprzet/konsole/",
                    last_value: 1,
                }),
            ],
        },
        Shop {
            title: "Morele",
            scraper: &(CountStringScraper {
                client: &client,
                string_to_count: "PRODUKT NIEDOSTĘPNY",
            }),
            pages: vec![
                (Page {
                    title: "Drive",
                    url: "https://www.morele.net/sony-playstation-5-5943281/",
                    last_value: 1,
                }),
                (Page {
                    title: "Digital",
                    url: "https://www.morele.net/sony-playstation-5-digital-5944164/",
                    last_value: 1,
                }),
            ],
        },
        Shop {
            title: "Morele",
            scraper: &(CountStringScraper {
                client: &client,
                string_to_count: "Produkt tymczasowo niedostępny",
            }),
            pages: vec![
                (Page {
                    title: "Drive",
                    url: "https://www.oleole.pl/konsole-playstation-5/sony-konsola-playstation-5-ps5-blu-ray-4k.bhtml",
                    last_value: 1,
                }),
                (Page {
                    title: "Digital",
                    url: "https://www.oleole.pl/konsole-playstation-5/sony-konsola-playstation-5-edycja-digital-ps5.bhtml",
                    last_value: 1,
                }),
                (Page {
                    title: "Index",
                    url: "https://www.oleole.pl/konsole-playstation-5.bhtml",
                    last_value: 2,
                }),
            ],
        },
        Shop {
            title: "Euro",
            scraper: &(CountStringScraper {
                client: &client,
                string_to_count: r#"status: "9""#,
            }),
            pages: vec![
                (Page {
                    title: "Drive",
                    url: "https://www.euro.com.pl/konsole-playstation-5/sony-konsola-playstation-5-ps5-blu-ray-4k.bhtml",
                    last_value: 2,
                }),
                (Page {
                    title: "Digital",
                    url: "https://www.euro.com.pl/konsole-playstation-5/sony-konsola-playstation-5-edycja-digital-ps5.bhtml",
                    last_value: 2,
                }),
                (Page {
                    title: "Index",
                    url: "https://www.euro.com.pl/konsole-playstation-5.bhtml",
                    last_value: 4,
                }),
            ],
        },
        Shop {
            title: "MediaMarkt",
            scraper: &(CountStringScraper {
                client: &client,
                string_to_count: "niedostępny",
            }),
            pages: vec![
                (Page {
                    title: "Drive",
                    url: "https://mediamarkt.pl/konsole-i-gry/konsola-sony-playstation-5",
                    last_value: 2,
                }),
                (Page {
                    title: "Digital",
                    url: "https://mediamarkt.pl/konsole-i-gry/konsola-sony-playstation-5-digital-edition",
                    last_value: 1,
                }),
                (Page {
                    title: "Digital + pad",
                    url: "https://mediamarkt.pl/konsole-i-gry/konsola-sony-playstation-5-digital-edition-dodatkowy-kontroler-dualsense",
                    last_value: 1,
                }),
            ],
        },
        Shop {
            title: "MediaMarkt - index",
            scraper: &(CountStringScraper {
                client: &client,
                string_to_count: "Nie znaleziono produktów",
            }),
            pages: vec![
                (Page {
                    title: "Index",
                    url: "https://mediamarkt.pl/konsole-i-gry/playstation-5/konsole-ps5",
                    last_value: 1,
                }),
            ],
        },
        Shop {
            title: "Neonet - product",
            scraper: &(CountStringScraper {
                client: &client,
                string_to_count: "UNPUBLISHED",
            }),
            pages: vec![
                (Page {
                    title: "Digital + pad",
                    url: "https://www.neonet.pl/graphql?query=query%20resolveUrl%7BurlResolver(url:%22/konsole-i-gry/playstation-5-digital-dualsense.html%22,search:%22%22)%7Btype%7D%0A%7D%0A&v=2.60.0",
                    // view_url: "https://www.neonet.pl/konsole-i-gry/playstation-5-digital-dualsense.html",
                    last_value: 1,
                }),
            ],
        },
        Shop {
            title: "Neonet - category",
            scraper: &(CountStringScraper {
                client: &client,
                string_to_count: "[]",
            }),
            pages: vec![
                (Page {
                    title: "Category index",
                    url: "https://www.neonet.pl/graphql?query=query%20msProducts%7BmsProducts(filter:%7Bskus:%5B100345611%5D%7D)%7Bitems_ids%7D%7D&v=2.60.0",
                    // view_url: "https://www.neonet.pl/konsole-i-gry/sony-playstation-5.html",
                    last_value: 1,
                }),
            ],
        },
        Shop {
            title: "Neonet - landing",
            scraper: &(CountStringScraper {
                client: &client,
                string_to_count: "[100346668,100346671]",
            }),
            pages: vec![
                (Page {
                    title: "Landing Page index",
                    url: "https://www.neonet.pl/graphql?query=query%20msProducts%7BmsProducts(filter:%7Blp_module_id:7198%7D)%7Bitems_ids%7D%7D&v=2.60.0",
                    // view_url: "https://www.neonet.pl/lpage/premiera-playstation5.html?kwplcm=banner_category",
                    last_value: 1,
                }),
            ],
        },
        Shop {
            title: "Neonet - offer",
            scraper: &(CountStringScraper {
                client: &client,
                string_to_count: "niedostępny",
            }),
            pages: vec![
                (Page {
                    title: "Digital",
                    url: "https://www.komputronik.pl/product/701048/sony-playstation-5-digital.html",
                    last_value: 1,
                }),
                (Page {
                    title: "Drive",
                    url: "https://www.komputronik.pl/product/701046/sony-playstation-5.html",
                    last_value: 1,
                }),
                (Page {
                    title: "Index",
                    url: "https://www.komputronik.pl/category/18885/konsole-playstation-5.html",
                    last_value: 2,
                }),
            ],
        },
        Shop {
            title: "Matrixmedia",
            scraper: &(CountStringScraper {
                client: &client,
                string_to_count: "Przepraszamy, ale wybrana strona nie może zostać znaleziona.",
            }),
            pages: vec![
                (Page {
                    title: "Digital + pad",
                    url: "https://matrixmedia.pl/zestaw-sony-playstation-5-dodatkowy-kontroler-dualsense-wireless-2-gry-karta-ps-plus.html",
                    last_value: 1,
                }),
            ],
        },
    ];

    // let futures = shops.iter().map(|shop| shop.scrape_pages());
    for shop in shops.iter() {
        shop.scrape_pages().await;
    }

    // stream::iter()
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
