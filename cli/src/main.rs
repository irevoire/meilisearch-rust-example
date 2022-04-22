use futures::executor::block_on;
use lazy_static::lazy_static;
use meilisearch_sdk::{client::*, document::*};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::{stdin, Read};

lazy_static! {
    /// This is an example for using doc comment attributes
    static ref CLIENT:Client = Client::new("http://localhost:7700", "masterKey");
}

fn main() {
    block_on(async move {
        build_index().await;
        loop {
            println!("Enter a search query or type \"q\" or \"quit\" to quit:");
            let mut input_string = String::new();
            stdin()
                .read_line(&mut input_string)
                .ok()
                .expect("Failed to read line");
            match input_string.trim() {
                "quit" | "q" | "" => {
                    println!("exiting...");
                    break;
                }
                _ => {
                    search(input_string.trim()).await;
                }
            }
        }
    })
}

async fn search(query: &str) {
    let query_results = CLIENT
        .index("clothes")
        .search()
        .with_query(query)
        .execute::<ClothesDisplay>()
        .await
        .unwrap()
        .hits;
    for clothes in query_results {
        println!("{:?}", clothes.result);
    }
}
/*
TODO:
Set display settings,
Ranking results
sort by price
add filter?
*/
async fn build_index() {
    // reading and parsing the filed
    let mut file = File::open("../assets/clothes.json").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let clothes: Vec<Clothes> = serde_json::from_str(&content).unwrap();
    let displayed_attributes = ["article", "cost", "size", "pattern"];
    let ranking_rules = ["words", "typo", "attribute", "exactness", "rank:desc"];
    let mut synonyms = std::collections::HashMap::new();
    synonyms.insert(
        String::from("sweater"),
        vec![String::from("sweatshirt"), String::from("long-sleeve")],
    );
    synonyms.insert(String::from("sweat pants"), vec![String::from("joggers")]);
    synonyms.insert(
        String::from("t-shirt"),
        vec![String::from("tees"), String::from("tshirt")],
    );

    // what kind of
    CLIENT
        .index("clothes")
        .set_synonyms(&synonyms)
        .await
        .unwrap();
    CLIENT
        .index("clothes")
        .add_or_update(&clothes, Some("id"))
        .await
        .unwrap();
    let _ = CLIENT
        .index("clothes")
        .set_displayed_attributes(displayed_attributes);
    let _ = CLIENT
        .index("clothes")
        .set_ranking_rules(&ranking_rules)
        .await
        .unwrap();
}

#[derive(Serialize,Deserialize, Debug)]
pub struct Clothes {
    id: usize,
    seaon: String,
    article: String,
    cost: f32,
    size: String,
    pattern: String,
}

impl Document for Clothes {
    type UIDType = usize;

    fn get_uid(&self) -> &Self::UIDType {
        &self.id
    }
}

#[derive(Serialize,Deserialize, Debug)]
pub struct ClothesDisplay {
    article: String,
    cost: f32,
    size: String,
    pattern: String,
}

impl fmt::Display for ClothesDisplay {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(
            f,
            "article {},\n price {},\n size {},\n pattern {}",
            self.article, self.cost, self.size, self.pattern
        )
    }
}
