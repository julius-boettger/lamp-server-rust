use crate::constants;
use std::sync::OnceLock; // thread-safe and can only be written to once

// global instance which will receive values from config file
pub static INSTANCE: OnceLock<Struct> = OnceLock::new();

// functions for convenience
pub fn api_key() -> String { INSTANCE.get().unwrap().api_key.clone() }
pub fn device() -> String { INSTANCE.get().unwrap().device.clone() }
pub fn model() -> String { INSTANCE.get().unwrap().model.clone() }

#[derive(Debug, serde::Deserialize)]
pub struct Struct {
    #[serde(rename(deserialize = "govee_api_key"))]
    pub api_key: String,
    #[serde(rename(deserialize = "govee_device"))]
    pub device: String,
    #[serde(rename(deserialize = "govee_model"))]
    pub model: String,
}

/// panics with error messages if valid `Struct` can't be retrieved
pub fn from_file() -> Struct {
    let mut path = dirs_next::config_dir()
        .expect("path to config file could not be determined, which means your operating system is not supported.\n");
    path.push(constants::CONFIG_FILE_NAME);

    // read file contents
    let yaml_config = std::fs::read_to_string(path.clone()).expect(format!(
        "config file could not be read from {}.\n\
        see the README for a template.\n", path.to_str().unwrap()
    ).as_str());
    
    // parse yaml string to struct
    return serde_yaml::from_str(&yaml_config).expect(format!(
        "config file at {} could not be parsed.\n\
        see the README for a template.\n", path.to_str().unwrap()
    ).as_str());
}