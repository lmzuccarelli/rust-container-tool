use std::fs::File;
use std::io::Read;
use crate::Path;
use crate::api::schema::*;

pub fn get_credentials() -> Result<String, Box<dyn std::error::Error>> {
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

pub fn parse_json_creds(data: String) -> Result<String, Box<dyn std::error::Error>> {
    // Parse the string of data into serde_json::Value.
    let creds: Root = serde_json::from_str(&data)?;
    Ok(creds.auths.registry_redhat_io.auth)
}

pub fn parse_json_token(data: String) -> Result<String, Box<dyn std::error::Error>> {
    // Parse the string of data into serde_json::Value.
    let root: Token = serde_json::from_str(&data)?;
    Ok(root.access_token)
}

pub fn parse_json_manifest(data: String) -> Result<ManifestSchema, Box<dyn std::error::Error>> {
    // Parse the string of data into serde_json::Value.
    let root: ManifestSchema = serde_json::from_str(&data)?;
    Ok(root)
}

pub async fn get_token(
    url: String,
    user: String,
    password: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let pwd: Option<String> = Some(password);
    let body = client
        .get(url)
        .basic_auth(user, pwd)
        .send()
        .await?
        .text()
        .await?;
    Ok(body)
}
