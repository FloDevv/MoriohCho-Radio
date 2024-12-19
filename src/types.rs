// News feed types

use serde::Deserialize;

#[derive(Debug)]
pub struct RssItem {
    pub title: String,
    pub description: String,
    pub link: String,
    pub date: String,
}

pub struct Article {
    pub title: String,
    pub content: String,
    pub source: String,
    pub date: String,
    pub description: String,
}

// Weather types
#[derive(Deserialize, Debug)]
pub struct GeocodingResponse {
    pub results: Vec<GeocodingResult>,
}

#[derive(Deserialize, Debug)]
pub struct GeocodingResult {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Deserialize, Debug)]
pub struct CurrentWeather {
    pub time: String,
    pub temperature: f64,
    pub weathercode: u8,
}

#[derive(Deserialize, Debug)]
pub struct Hourly {
    pub temperature_2m: Vec<f64>,
}

#[derive(Deserialize, Debug)]
pub struct WeatherResponse {
    pub current_weather: CurrentWeather,
    pub hourly: Hourly,
    #[serde(default)]
    pub city: String,
}
