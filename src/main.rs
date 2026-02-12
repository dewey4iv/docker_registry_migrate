use std::error::Error;

use serde::{Deserialize, Serialize};
use tokio::{
    fs::{File, read_to_string}, io::AsyncWriteExt, process::Command
};

pub type BoxedError = Box<dyn Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), BoxedError> {
    println!("reading config...");

    let config = read_config().await?;

    println!("migrating:\n\t{} -> {}", config.source, config.destination);

    println!("starting migrations...");

    let mut log = File::options()
        .write(true)
        .create(true)
        .truncate(false)
        .open("./docker_registry_migrate.log")
        .await?;

    migrate(&config, &mut log).await?;

    Ok(())
}

pub async fn migrate(config: &Config, log: &mut File) -> Result<(), BoxedError> {
    let default_platforms = config.platforms.join(",");

    for Image { image, versions, platforms } in &config.images {
        for version in versions {
            println!("migrating: {image}:{version}");

            let src_url = format!("{}/{}:{}", config.source, image, version);

            let dst_url = format!("{}/{}:{}", config.destination, image, version);

            let platforms_str = match platforms {
                Some(platforms) => &platforms.join(","),
                None => &default_platforms,
            };

            docker_cmd(&src_url, &dst_url, platforms_str, log).await?;
        }
    }

    Ok(())
}

pub async fn docker_cmd(src: &str, dst: &str, platforms: &str, log: &mut File) -> Result<(), BoxedError> {
    let output = Command::new("docker")
        .arg("buildx")
        .arg("imagetools")
        .arg("create")
        .arg("--tag")
        .arg(dst)
        .arg(src)
        .arg("--platform")
        .arg(platforms)
        .output()
        .await?;

    println!("{:?}", String::from_utf8(output.stdout.clone())?);
    println!("{:?}", String::from_utf8(output.stderr.clone())?);

    log.write_all(&output.stdout).await?;
    log.write_all(&output.stderr).await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    source: String,
    destination: String,
    platforms: Vec<String>,
    images: Vec<Image>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    image: String,
    versions: Vec<String>,
    platforms: Option<Vec<String>>,
}

pub async fn read_config() -> Result<Config, BoxedError> {
    let raw = read_to_string("./config.json").await?;

    let config = serde_json::from_str(&raw)?;

    Ok(config)
}
