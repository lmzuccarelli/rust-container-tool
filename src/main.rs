use base64::{engine::general_purpose, Engine as _};
use futures::{stream, StreamExt};
use reqwest::Client;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use tokio;
use flate2::read::GzDecoder;
use tar::Archive;
//use std::io::prelude::*;
use std::path::Path;
use std::str;
use std::env;


// define modules
mod log;
mod api;
mod auth;

use log::logging::*;
use api::schema::*;
use auth::credentials::*;

#[tokio::main]
async fn main() {
    // global variables can be tricky in rust :)
    const REALM_URL: &str = "https://sso.redhat.com/auth/realms/rhcc/protocol/redhat-docker-v2/auth?service=docker-registry&client_id=curl&scope=repository:rhel:pull";
    const MANIFESTS_URL: &str =
        "https://registry.redhat.io/v2/redhat/redhat-operator-index/manifests/v4.12";
    const MANIFEST_JSON_DIR: &str = "working-dir/rhopi/manifest.json";
    const BLOBS_URL: &str = "https://registry.redhat.io/v2/redhat/redhat-operator-index/blobs/";

    // check if the rhopi directory exists
    if !Path::new("working-dir/rhopi/blobs/sha256").exists() {
        // get creds from $XDG_RUNTIME_DIR
        let creds = get_credentials().unwrap();
        // parse the json data
        let rhauth = parse_json_creds(creds).unwrap();
        // decode to base64
        let bytes = general_purpose::STANDARD.decode(rhauth).unwrap();

        let s = match str::from_utf8(&bytes) {
            Ok(v) => v,
            Err(e) => panic!("ERROR: invalid UTF-8 sequence: {}", e),
        };
        // get user and password form json
        let user = s.split(":").nth(0).unwrap();
        let pwd = s.split(":").nth(1).unwrap();
        // call the realm url get a token with the creds
        let res = get_token(REALM_URL.to_string(), user.to_string(), pwd.to_string())
            .await
            .unwrap();
        let token = parse_json_token(res).unwrap();
        // use token to get manifest
        let manifest = get_manifest(MANIFESTS_URL.to_string(), token.clone())
            .await
            .unwrap();

        // create the full path
        fs::create_dir_all("working-dir/rhopi/blobs/sha256").expect("unable to create directory");
        fs::write(MANIFEST_JSON_DIR, manifest.clone()).expect("unable to write file");
        let res = parse_json_manifest(manifest).unwrap();
        get_blobs(BLOBS_URL.to_string(), token, res.fs_layers).await;
        log_info("completed image index download");
    } else {
        log_info("catalog index exists nothing to do");
    }
    // check if the cache directory exists
    if !Path::new("working-dir/cache/rhopi").exists() {
        // create the cache directory
        fs::create_dir_all("working-dir/cache/rhopi").expect("unable to create directory");
        untar_layers("working-dir/rhopi/blobs/sha256".to_string());
        log_info("completed untar of layers");
    } else {
        log_info("cache exists nothing to do");
    }
    let dir = find_dir("working-dir/cache/rhopi".to_string(), "configs".to_string());
    log_info(&format!("full path for directory 'configs' {} ",&dir));
    if dir != "" {
        list_components("operator".to_string(),dir);
    } else {
        log_error("configs directory not found");
    }
}

// get manifest
async fn get_manifest(url: String, token: String) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut header_bearer: String = "Bearer ".to_owned();
    header_bearer.push_str(&token);
    let body = client
        .get(url)
        .header("Accept", "application/vnd.oci.image.manifest.v1+json")
        .header("Content-Type", "application/json")
        .header("Authorization", header_bearer)
        .send()
        .await?
        .text()
        .await?;
    Ok(body)
}

async fn get_blobs(url: String, token: String, layers: Vec<FsLayer>) {
    const PARALLEL_REQUESTS: usize = 8;
    const BLOBS_DIR: &str = "working-dir/rhopi/blobs/sha256/";

    let client = Client::new();
    let mut header_bearer: String = "Bearer ".to_owned();
    header_bearer.push_str(&token);

    // remove all duplicates in FsLayer
    let mut images = Vec::new();
    let mut seen = HashSet::new();
    for img in layers {
        if !seen.contains(&img.blob_sum) {
            seen.insert(img.blob_sum.clone());
            images.push(img.blob_sum);
        }
    }

    let fetches = stream::iter(images.into_iter().map(|blob| {
        let client = client.clone();
        let url = url.clone();
        let header_bearer = header_bearer.clone();
        async move {
            match client
                .get(url + &blob)
                .header("Authorization", header_bearer)
                .send()
                .await
            {
                Ok(resp) => match resp.bytes().await {
                    Ok(bytes) => {
                        let blob = blob.split(":").nth(1).unwrap();
                        fs::write(BLOBS_DIR.to_string() + &blob, bytes.clone())
                            .expect("unable to write blob");
                        let msg = format!("writing blob {}",blob);
                        log_info(&msg);
                    }
                    Err(_) => {
                        let msg = format!("reading blob {}",&blob);
                        log_error(&msg);
                    }
                },
                Err(_) => {
                    let msg = format!("downloading blob {}",&blob);
                    log_error(&msg);
                }
            }
        }
    }))
    .buffer_unordered(PARALLEL_REQUESTS)
    .collect::<Vec<()>>();
    log_info("downloading blobs...");
    fetches.await;
}

fn untar_layers(dir: String)  {
    // change to the blobs/sha256 directory
    env::set_current_dir(&dir).expect("could not set current directory");
    // read directory, iterate each file and untar
    let paths = fs::read_dir(".").unwrap();
    for path in paths {
        let entry = path.expect("could not resolve file entry");
        let file = entry.path();
        let tar_gz = File::open(file.clone()).expect("could not open file");
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        // should always be a sha256 string
        let tar_dir = file.into_os_string().into_string().unwrap();
        let msg = format!("untarring file {} ",&tar_dir[2..]);
        log_info(&msg);
        match archive.unpack("../../../cache/rhopi/".to_string() + &tar_dir[2..10]) {
            Ok(arch) => arch,
            Err(error) => {
                let msg = format!("skipping this error : {} ",&error.to_string());
                log_warn(&msg);
            }
        };
    }
}

// find a specifc directory in the untar layers
fn find_dir(dir: String,name: String) -> String {
    let paths = fs::read_dir(&dir).unwrap();
    // for both release & operator image indexes 
    // we know the layer we are looking for is only 1 level 
    // down from the parent
    for path in paths {
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
    return "".to_string()
}

// list all components in the current image index
fn list_components(ctype: String,dir: String) {
    let paths = fs::read_dir(&dir).unwrap();
    for path in paths {
        let entry = path.expect("could not resolve path entry");
        let dir_name = entry.path();
        let str_dir = dir_name.into_os_string().into_string().unwrap();
        let res = str_dir.split("/");
        let name = format!("{} => {}",ctype,res.into_iter().last().unwrap());
        log_hi(&name);
    }
}


