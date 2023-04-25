use clap::Parser;
use std::fs;
use std::path::Path;
use std::str;
use tokio;

// define modules
mod api;
mod auth;
mod config;
mod image;
mod list;
mod log;
mod manifests;

use api::schema::*;
use auth::credentials::*;
use config::read::*;
use image::copy::*;
use list::components::*;
use log::logging::*;
use manifests::catalogs::*;

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let filter = args.filter.as_ref().unwrap().to_string();
    log_info(&format!(
        "rust-container-tools {} {} {}",
        args.config, args.dir, filter
    ));
    // Parse the config serde_yaml::ImageSetConfig.
    let config = load_config(args.config).unwrap();
    let isc = parse_yaml_config(config).unwrap();
    log_debug(&format!("{:#?}", isc.mirror.platform));

    // global variables can be tricky in rust :)
    const MANIFESTS_URL: &str =
        "https://registry.redhat.io/v2/redhat/certified-operator-index/manifests/v4.12";
    const MANIFEST_JSON_DIR: &str = "working-dir/certified/manifest.json";
    const BLOBS_URL: &str = "https://registry.redhat.io/v2/redhat/certified-operator-index/blobs/";

    // check if the directory exists
    if !Path::new("working-dir/certified/blobs/sha256").exists() {
        let token = get_token().await;
        // use token to get manifest
        let manifest = get_manifest(MANIFESTS_URL.to_string(), token.clone())
            .await
            .unwrap();

        // create the full path
        fs::create_dir_all("working-dir/certified/blobs/sha256")
            .expect("unable to create directory");
        fs::write(MANIFEST_JSON_DIR, manifest.clone()).expect("unable to write file");
        let res = parse_json_manifest(manifest).unwrap();
        get_blobs(BLOBS_URL.to_string(), token, res.fs_layers).await;
        log_info("completed image index download");
    } else {
        log_info("catalog index exists nothing to do");
    }
    // check if the cache directory exists
    if !Path::new("working-dir/cache/certified").exists() {
        // create the cache directory
        fs::create_dir_all("working-dir/cache/certified").expect("unable to create directory");
        untar_layers("working-dir/certified/blobs/sha256".to_string()).await;
        log_info("completed untar of layers");
    } else {
        log_info("cache exists nothing to do");
    }
    if args.action == "list" {
        let dir = find_dir(
            "working-dir/cache/certified".to_string(),
            "configs".to_string(),
        )
        .await;
        log_info(&format!("full path for directory 'configs' {} ", &dir));
        if dir != "" {
            list_components("operator".to_string(), dir, filter).await;
        } else {
            log_error("configs directory not found");
        }
    }
}
