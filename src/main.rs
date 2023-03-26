use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use base64::{Engine as _, engine::general_purpose};
use std::str;
use futures::{stream, StreamExt};
use reqwest::Client;
use tokio;


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestSchema {
    pub tag: String,
    pub name: String,
    pub architecture: String,
    pub schema_version: i64,
    pub history: Vec<History>,
    pub fs_layers: Vec<FsLayer>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct History {
    #[serde(rename = "v1Compatibility")]
    pub v1compatibility: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsLayer {
    pub blob_sum: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub token: String,
    #[serde(rename = "access_token")]
    pub access_token: String,
    #[serde(rename = "expires_in")]
    pub expires_in: i64,
    #[serde(rename = "issued_at")]
    pub issued_at: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub auths: Auths,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Auths {
    #[serde(rename = "cloud.openshift.com")]
    pub cloud_openshift_com: CloudOpenshiftCom,
    #[serde(rename = "quay.io")]
    pub quay_io: QuayIo,
    #[serde(rename = "registry.connect.redhat.com")]
    pub registry_connect_redhat_com: RegistryConnectRedhatCom,
    #[serde(rename = "registry.redhat.io")]
    pub registry_redhat_io: RegistryRedhatIo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudOpenshiftCom {
    pub auth: String,
    pub email: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuayIo {
    pub auth: String,
    pub email: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryConnectRedhatCom {
    pub auth: String,
    pub email: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryRedhatIo {
    pub auth: String,
    pub email: String,
}

#[tokio::main]
async fn main() {
    // global variables can be tricky in rust :)
    const REALM_URL: &str = "https://sso.redhat.com/auth/realms/rhcc/protocol/redhat-docker-v2/auth?service=docker-registry&client_id=curl&scope=repository:rhel:pull";
    const MANIFESTS_URL: &str = "https://registry.redhat.io/v2/redhat/redhat-operator-index/manifests/v4.11";
    const MANIFEST_JSON_DIR: &str = "working-dir/rhopi/manifest.json";
    const BLOBS_URL: &str = "https://registry.redhat.io/v2/redhat/redhat-operator-index/blobs/";

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
    let res = get_token(REALM_URL.to_string(),user.to_string(),pwd.to_string()).await.unwrap();
    let token = parse_json_token(res).unwrap();
    // use token to get manifest
    let manifest = get_manifest(MANIFESTS_URL.to_string(), token.clone()).await.unwrap();
    fs::write(MANIFEST_JSON_DIR, manifest.clone()).expect("unable to write file");
    let res = parse_json_manifest(manifest).unwrap();
    println!("manifest = {:#?} ", res);
    get_blobs(BLOBS_URL.to_string(), token, res.fs_layers).await;
}


fn get_credentials() -> Result<String,Box<dyn std::error::Error>>{
    // Create a path to the desired file
    let path = Path::new("/run/user/1000/containers/auth.json");
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok(s)
}

pub fn parse_json_creds(data: String) -> Result<String,Box<dyn std::error::Error>> {
    // Parse the string of data into serde_json::Value.
    let root: Root = serde_json::from_str(&data)?;
    Ok(root.auths.registry_redhat_io.auth)
}

pub fn parse_json_token(data: String) -> Result<String,Box<dyn std::error::Error>> {
    // Parse the string of data into serde_json::Value.
    let root: Token = serde_json::from_str(&data)?;
    Ok(root.access_token)
}

pub fn parse_json_manifest(data: String) -> Result<ManifestSchema,Box<dyn std::error::Error>> {
    // Parse the string of data into serde_json::Value.
    let root: ManifestSchema = serde_json::from_str(&data)?;
    Ok(root)
}


async fn get_token(url: String,user: String,password: String) -> Result<String,Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let pwd: Option<String> = Some(password);
    let body = client.get(url).basic_auth(user, pwd)
        .send()
        .await?
        .text()
        .await?;
    Ok(body)
}

async fn get_manifest(url: String,token: String) -> Result<String,Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut header_bearer: String = "Bearer ".to_owned();
    header_bearer.push_str(&token);    
    let body = client.get(url)
        .header("Accept", "application/vnd.oci.image.manifest.v1+json")
        .header("Content-Type", "application/json")
        .header("Authorization", header_bearer)
        .send()
        .await?
        .text()
        .await?;
    Ok(body)
}

async fn get_blobs(url: String, token: String,layers: Vec<FsLayer>) {
    const PARALLEL_REQUESTS: usize = 8;
    const BLOBS_DIR: &str = "working-dir/rhopi/blobs/sha256/";

    let client = Client::new();
    let mut header_bearer: String = "Bearer ".to_owned();
    header_bearer.push_str(&token);

    let fetches = stream::iter(
    layers.into_iter().map(|blob| {
        let client = client.clone(); 
        let url = url.clone();
        let header_bearer = header_bearer.clone();
        async move {
            match client.get(url+&blob.blob_sum).header("Authorization", header_bearer).send().await {
                        Ok(resp) => {
                    match resp.bytes().await {
                        Ok(bytes) => {
                            let blob = blob.blob_sum.split(":").nth(1).unwrap();
                            fs::write(BLOBS_DIR.to_string()+&blob, bytes.clone()).expect("unable to write blob");
                            println!("INFO: writing response to {}", blob);
                        }
                        Err(_) => println!("ERROR: reading {}", blob.blob_sum),
                    }
                }
                Err(_) => println!("ERROR: downloading {}", blob.blob_sum),
            }
        }
    })).buffer_unordered(PARALLEL_REQUESTS).collect::<Vec<()>>();
    println!("INFO: waiting...");
    fetches.await;
}
