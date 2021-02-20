use yaml_rust::{YamlLoader, Yaml};
use std::env::{var, VarError};
use crate::server::error::AError;

#[macro_use]
lazy_static::lazy_static! {
    static ref YAML:Vec<Yaml> = {
        let config = std::fs::read_to_string("./config/application.yaml").unwrap();
        let result = YamlLoader::load_from_str(&config).unwrap();
        result
    };
}



pub fn get_i64(key: &str) -> Option<i64> {
    (*YAML)[0][key].as_i64()
}

pub fn get_string(key: &str) -> Option<&str> {
    (*YAML)[0][key].as_str()
}

pub fn get_env(key: &str) -> Result<String, AError> {
    let name = &key.to_owned()[2..key.len() - 1];
    Ok(match var(name) {
        Ok(value) => {
            value
        }
        Err(_) => {
            return Err(AError::service(format!("环境变量: {} 找不到", name).as_str()));
        }
    })
}