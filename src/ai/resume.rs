use reqwest::Client;
use serde_json::{ json, Value };
use crate::{ config::types::Config, types::WeatherResponse };

pub async fn ai_resume(
    weather: &WeatherResponse,
    articles_text: &str,
    client: &Client,
    config: &Config
) -> Result<String, Box<dyn std::error::Error>> {
    let weather_info: String = format!(
        "Current weather at {} :\nTime: {}\nTemperature: {}°C\nConditions: {}\n{}",
        weather.city,
        weather.current_weather.time,
        weather.current_weather.temperature,
        weather.current_weather.get_weather_description(),
        weather.get_day_forecast()
    );
    let payload: Value =
        json!({
        "model": "llama-3.3-70b-versatile",
        "messages": [
            {
                "role": "system",
                "content": format!(
                    "You are a skilled journalist working for Morioh-cho Radio's morning news segment. Write in plain text, no markdown format. After the good morning greeting, tell about the meteo of today and summarize the news in a clear and concise way. End with a Have a great day !. You speak and write in {}.",
                    &config.language
                )
            },
            {
                "role": "user",
                "content": format!(
                    "Please provide a comprehensive summary of these news articles:\n{}{}",
                    weather_info,
                    articles_text
                )
            }
        ],
        "temperature": 0.3,
        "max_tokens": 8000,
        "top_p": 0.8,
        "stream": false
    });

    let response: reqwest::Response = client
        .post(&config.api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", &config.api_key))
        .json(&payload)
        .send().await?;

    if !response.status().is_success() {
        let status: reqwest::StatusCode = response.status();
        let error_text: String = response.text().await?;
        return Err(format!("API error: {} - {}", status, error_text).into());
    }

    let body: Value = response.json().await?;
    let content: String = body["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Failed to extract content")?
        .to_string();

    Ok(content)
}
