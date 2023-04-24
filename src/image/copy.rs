use futures::{stream, StreamExt};
use reqwest::Client;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use flate2::read::GzDecoder;
use tar::Archive;
use std::str;
use std::env;

use crate::api::schema::*;
use crate::log::logging::*;

// get manifest
pub async fn get_manifest(url: String, token: String) -> Result<String, Box<dyn std::error::Error>> {
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

pub async fn get_blobs(url: String, token: String, layers: Vec<FsLayer>) {
    const PARALLEL_REQUESTS: usize = 8;
    const BLOBS_DIR: &str = "working-dir/certified/blobs/sha256/";

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

pub async fn untar_layers(dir: String)  {
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
        // we are really interested in either the configs or releas-images directories
        match archive.unpack("../../../cache/certified/".to_string() + &tar_dir[2..10]) {
            Ok(arch) => arch,
            Err(error) => {
                let msg = format!("skipping this error : {} ",&error.to_string());
                log_warn(&msg);
            }
        };
    }
}

