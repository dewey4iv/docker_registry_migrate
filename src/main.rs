use std::{error::Error, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::{fs::read_to_string, process::Command, time::sleep};

pub type BoxedError = Box<dyn Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), BoxedError> {
    println!("reading config...");

    let config = read_config().await?;

    println!("migrating:\n\t{} -> {}", config.source, config.destination);

    println!("starting migrations...");

    migrate(&config).await?;

    Ok(())
}

pub async fn migrate(config: &Config) -> Result<(), BoxedError> {
    for Image { image, versions } in &config.images {
        for version in versions {
            println!("migrating: {image}:{version}");

            let src_url = format!("{}/{}:{}", config.source, image, version);
            let dst_url = format!("{}/{}:{}", config.destination, image, version);

            docker_cmd(&src_url, &dst_url).await?;
        }
    }

    Ok(())
}

pub async fn docker_cmd(src: &str, dst: &str) -> Result<(), BoxedError> {
    // let output = Command::new("docker")
    //     .arg("buildx")
    //     .arg("imagetools")
    //     .arg("create")
    //     .arg("--tag")
    //     .arg(dst)
    //     .arg(src)
    //     .output().await?;
    //
    //     println!("{output:?}");

    sleep(Duration::from_secs(2)).await;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    source: String,
    destination: String,
    images: Vec<Image>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    image: String,
    versions: Vec<String>,
}

pub async fn read_config() -> Result<Config, BoxedError> {
    let raw = read_to_string("./config.json").await?;

    let config = serde_json::from_str(&raw)?;

    Ok(config)
}
