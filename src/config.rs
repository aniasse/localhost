use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub host: String,
    pub ports: Vec<u16>,
    pub error_pages: HashMap<String, String>,
    pub routes: HashMap<String, Route>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Route {
    pub methods: Vec<String>,
    pub root: String,
    pub cgi: Option<String>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_yaml::from_str(&content).map_err(|e| e.to_string())
    }
}