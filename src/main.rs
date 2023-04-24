use std::fs;
use tokio;
use std::path::Path;
use std::str;
use clap::Parser;

// define modules
mod api;
mod auth;
mod config;
mod image;
mod log;
mod manifests;

use api::schema::*;
use auth::credentials::*;
use config::read::*;
use image::copy::*;
use log::logging::*;
use manifests::catalogs::*;

#[tokio::main]
async fn main() {

    let args = Cli::parse();
    let filter = args.filter.as_ref().unwrap().to_string();
    log_info(&format!("rust-container-tools {} {} {}", args.config,args.dir,filter));
    // Parse the config serde_yaml::ImageSetConfig.
    let config = load_config(args.config).unwrap();
    let isc = parse_yaml_config(config).unwrap(); 
    log_debug(&format!("{:#?}",isc.mirror.platform));

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
        fs::create_dir_all("working-dir/certified/blobs/sha256").expect("unable to create directory");
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
        let dir = find_dir("working-dir/cache/certified".to_string(), "configs".to_string()).await;
        log_info(&format!("full path for directory 'configs' {} ",&dir));
        if dir != "" {
            list_components("operator".to_string(),dir,filter).await;
        } else {
            log_error("configs directory not found");
        }
    }
}

// find a specifc directory in the untar layers
async fn find_dir(dir: String,name: String) -> String {
    // return to current working dir
    // env::set_current_dir("../../../../").expect("could not set current directory");
    let paths = fs::read_dir(&dir);
    // for both release & operator image indexes 
    // we know the layer we are looking for is only 1 level 
    // down from the parent
    match paths {
        Ok(res_paths) => {
            for path in res_paths {
                let entry = path.expect("could not resolve path entry");
                let file = entry.path();
                // go down one more level
                let sub_paths = fs::read_dir(file).unwrap();
                for sub_path in sub_paths {
                    let sub_entry = sub_path.expect("could not resolve sub path entry");
                    let sub_name = sub_entry.path();
                    let str_dir = sub_name.into_os_string().into_string().unwrap();
                    if str_dir.contains(&name) {
                        return str_dir;
                    }
                }
            }
        },
        Err(error) => {
            let msg = format!("{} ",error);
            log_warn(&msg); 
        }
    }
    return "".to_string();
}

// list all components in the current image index
async fn list_components(ctype: String,dir: String, filter: String) {
    let paths = fs::read_dir(&dir).unwrap();
    for path in paths {
        let entry = path.expect("could not resolve path entry");
        let dir_name = entry.path();
        let str_dir = dir_name.into_os_string().into_string().unwrap();
        let res = str_dir.split("/");
        let name = format!("{} => {}",ctype,res.into_iter().last().unwrap());
        if filter != "all" && name.contains(&filter) {
            let dc = read_operator_catalog(str_dir);
            log_hi(&name);
            list_channel_info(dc.unwrap());
            break;
        } else if filter == "all" {
            log_hi(&name);
            //list_channel_info(dc.iter());
        }
    }
}

fn list_channel_info(dc: serde_json::Value) {
    let dc: Vec<DeclarativeConfig> = serde_json::from_value(dc).unwrap();
    for x in dc {
        /*for t in x {
            if t.default_channel != None {
                let channel = t.default_channel.clone().unwrap();
                log_lo(&format!("   defaultChannel => {}",channel));
            }
            if t.schema == "olm.channel" {
                log_lo(&format!("   channel => {}", t.name));
            }
            if t.schema == "olm.bundle" {
                log_mid(&format!("      bundle => {}", t.name));
            }
        }*/

        println!("LMZ DEBUG {:#?}",x);
    }
}

