use crate::api::schema::*;
use crate::log::logging::*;
use crate::manifests::catalogs::*;
use std::fs;

// list all components in the current image index
pub async fn list_components(ctype: String, dir: String, filter: String) {
    let paths = fs::read_dir(&dir).unwrap();

    if filter != "all" {
        let dc = read_operator_catalog(dir + &"/".to_string() + &filter);
        log_hi(&filter);
        list_channel_info(dc.unwrap());
    } else {
        for path in paths {
            let entry = path.expect("could not resolve path entry");
            let dir_name = entry.path();
            let str_dir = dir_name.into_os_string().into_string().unwrap();
            let res = str_dir.split("/");
            let name = format!("{} => {}", ctype, res.into_iter().last().unwrap());
            let dc = read_operator_catalog(str_dir);
            log_hi(&name);
            list_channel_info(dc.unwrap());
        }
    }
}

// iterate through object and display values
pub fn list_channel_info(dc: serde_json::Value) {
    // attempt to read entries first
    // this is because some catalogs don't have entries and
    // this would cause a panic
    let de: Vec<DeclarativeEntries> = match serde_json::from_value(dc.clone()) {
        Ok(val) => val,
        Err(_) => {
            let ch = ChannelEntry {
                name: String::from("none"),
                skips: Some(String::from("no-skips")),
                skip_range: Some(String::from("no-skip_range")),
                replaces: Some(String::from("no-replaces")),
            };
            let v = vec![ch];
            let de = DeclarativeEntries { entries: Some(v) };
            let vde = vec![de];
            vde
        }
    };

    // read the rest of the declarative config
    let dc: Vec<DeclarativeConfig> = serde_json::from_value(dc).unwrap();

    // list channel, bundles and defaultChannel
    for x in dc {
        if x.default_channel != None {
            let channel = x.default_channel.clone().unwrap();
            log_lo(&format!("  defaultChannel => {}", channel));
        }
        if x.schema == "olm.channel" {
            log_lo(&format!("  channel => {}", x.name));
        }
        if x.schema == "olm.bundle" {
            log_mid(&format!("    bundle => {}", x.name));
        }
    }
    // lis tall entries
    for decl_entry in de.into_iter() {
        if let Some(channel_entry) = decl_entry.entries {
            for item in channel_entry {
                log_ex(&format!("    entry => {}", item.name));
            }
        }
    }
}
