
use std::fs::File;
use std::io::Read;
use std::error::Error;
use crate::api::schema::*;

// read_operator_catalog - simple function tha treads the specific catalog.json file
// and unmarshals it to DeclarativeConfig struct
pub fn read_operator_catalog(path: String) -> Result<serde_json::Value,Box<dyn Error>> {
    let catalog = path + &"/catalog.json".to_owned();
    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&catalog) {
        Err(why) => panic!("couldn't open {}: {}",catalog, why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    let res = s.replace(" ","");
    let updated_json = "{ \"overview\": [".to_string() + &res.replace("}\n{","},{") + &"]}".to_string();
    // Parse the string of data into serde_json::Vec<DeclarativeConfig>
    let root = match serde_json::from_str::<Catalog>(&updated_json) {
        Ok(val) => val,
        Err(error) => panic!("error {}",error),
    };
    //println!("DEBUG LMZ {:#?}",root);
    //let root: Vec<DeclarativeConfig> = serde_json::from_str(&updated_json)?;
    Ok(root.overview)
}
