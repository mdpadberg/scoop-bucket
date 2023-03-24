use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};
use sha256::digest_bytes;
use std::error::Error;
use tera::{Context, Tera};
#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref TEMPLATE: Tera = {
        match Tera::new("templates/*.json") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        }
    };
}

#[derive(Serialize, Debug)]
struct Template {
    version: String,
    url: String,
    hash: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct GithubApi {
    #[serde(rename = "tag_name")]
    version_number: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Asset {
    #[serde(rename = "browser_download_url")]
    url: String,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.github.com/repos/mdpadberg/multi-cf/releases/latest")
        .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36")
        .send()?
        .json::<GithubApi>()?;
    let mut template = Template {
        version: response.version_number,
        url: String::new(),
        hash: String::new(),
    };
    for asset in response.assets {
        if asset.url.contains("x86_64-pc-windows-gnu.zip") {
            let asset_as_bytes = client
                .get(&asset.url)
                .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36")
                .send()?
                .bytes()?;
            template.url = asset.url;
            template.hash = digest_bytes(&asset_as_bytes);
        }
    }
    println!(
        "{}",
        TEMPLATE.render("mcf.json", &Context::from_serialize(&template)?)?
    );
    Ok(())
}
