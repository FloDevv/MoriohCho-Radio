use crate::config::types::Config;

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    const CONFIG_STR: &str = include_str!("../../sources.json");
    let config: Config = serde_json::from_str(CONFIG_STR)?;
    Ok(config)
}
