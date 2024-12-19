use reqwest::Client;
use crate::types::{GeocodingResult, WeatherResponse, CurrentWeather, GeocodingResponse};

impl CurrentWeather {
    pub fn get_weather_description(&self) -> &'static str {
        match self.weathercode {
            0 => "Clear sky",
            1..=3 => "Partly cloudy",
            45 | 48 => "Foggy",
            51..=57 => "Drizzle",
            61..=67 => "Rain",
            71..=77 => "Snow",
            80..=86 => "Showers",
            95..=99 => "Thunderstorm",
            _ => "Unknown"
        }
    }
}

impl WeatherResponse {
    pub fn get_day_forecast(&self) -> String {
        let temps: &[f64] = &self.hourly.temperature_2m[..24];
        let (min_temp, max_temp) = temps.iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &temp|
                (min.min(temp), max.max(temp))
            );

        format!("{}: {}, {:.1}°C to {:.1}°C",
            self.city,
            self.current_weather.get_weather_description(),
            min_temp,
            max_temp
        )
    }
}

pub async fn fetch_weather(city: &str) -> Result<WeatherResponse, Box<dyn std::error::Error>> {
    let client: Client = Client::new();
    // Get coordinates
    let geo_url: String = format!("https://geocoding-api.open-meteo.com/v1/search?name={}&count=1", city);
    let coords: GeocodingResult = client.get(&geo_url)
        .send()
        .await?
        .json::<GeocodingResponse>()
        .await?
        .results
        .into_iter()
        .next()
        .ok_or("City not found")?;

    // Get weather
    let weather_url: String = format!(
        "https://api.open-meteo.com/v1/forecast?\
        latitude={}&longitude={}\
        &current_weather=true\
        &hourly=temperature_2m",
        coords.latitude,
        coords.longitude
    );

    let mut response: WeatherResponse = client.get(&weather_url)
        .send()
        .await?
        .json()
        .await?;

    response.city = city.to_string();
    Ok(response)
}
