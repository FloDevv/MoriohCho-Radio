use serde::{ Deserialize, Serialize };

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Source {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilterConfig {
    pub categories: Vec<String>,
    pub banned: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub news_sources: Vec<Source>,
    pub filter: FilterConfig,
    pub city: String,
    pub api_key: String,
    pub api_url: String,
    pub language: String,
}
