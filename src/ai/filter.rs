use reqwest::Client;
use serde_json::{ json, Value };
use std::sync::atomic::{ AtomicUsize, Ordering };

use crate::config::types::Config;
use crate::config::types::FilterConfig;
static MODEL_INDEX: AtomicUsize = AtomicUsize::new(0);
const MAX_RETRIES: u32 = 10;
const TIMEOUT_SECS: u64 = 10;

pub async fn aifilter(
    title: &str,
    description: &str,
    config: &Config,
    filter_config: &FilterConfig,
    client: &Client
) -> Result<bool, Box<dyn std::error::Error>> {
    let categories: String = filter_config.categories.join(", ");
    let models: Vec<&str> = vec![
        "gemma2-9b-it",
        "llama-3.1-70b-versatile",
        "llama-3.2-11b-vision-preview",
        "llama-3.3-70b-versatile"
    ];

    for attempt in 0..MAX_RETRIES {
        let current_model: &str = get_next_model(&models);
        let payload: Value = build_api_payload(current_model, &categories, title, description);

        if
            let Ok(Some(result)) = make_api_request(
                client,
                config,
                payload,
                current_model,
                attempt,
                title
            ).await
        {
            return Ok(result);
        }

        tokio::time::sleep(std::time::Duration::from_secs(2 * ((attempt + 1) as u64))).await;
    }
    Ok(false)
}

fn get_next_model<'a>(models: &'a [&'a str]) -> &'a str {
    let current_index: usize = MODEL_INDEX.fetch_add(1, Ordering::SeqCst) % models.len();
    models[current_index]
}

fn build_api_payload<'a>(model: &str, categories: &str, title: &str, description: &str) -> Value {
    json!({
        "model": model,
        "messages": [
            {
                "role": "assistant",
                "content": format!(
                    "You are a news filter You MUST respond with ONLY 'true' or 'false'\n\
                    RULES:\n\
                    1. Answer 'true' if content matches any category: {}\n\
                    2. Answer 'false' if no match is found",
                    categories
                )
            },
            {
                "role": "user",
                "content": format!(
                    "Evaluate if this content matches any category:\nTitle: {}\nDescription: {}",
                    title, description
                )
            }
        ],
        "temperature": 0.1,
        "max_tokens": 1,
        "top_p": 0.1,
    })
}

async fn make_api_request(
    client: &Client,
    config: &Config,
    payload: Value,
    current_model: &str,
    attempt: u32,
    title: &str
) -> Result<Option<bool>, Box<dyn std::error::Error>> {
    match
        tokio::time::timeout(
            std::time::Duration::from_secs(TIMEOUT_SECS),
            client
                .post(&config.api_url)
                .header("Authorization", format!("Bearer {}", &config.api_key))
                .json(&payload)
                .send()
        ).await
    {
        Ok(response_result) =>
            match response_result {
                Ok(response) => {
                    if !response.status().is_success() {
                        let status: reqwest::StatusCode = response.status();
                        let error_text: String = response.text().await?;
                        #[cfg(debug_assertions)]
                        eprintln!(
                            "API error with model {} on attempt {}: {} - {}",
                            current_model,
                            attempt + 1,
                            status,
                            error_text
                        );

                        if status.is_client_error() {
                            return Ok(None);
                        }
                        return Ok(None);
                    }

                    match response.json::<Value>().await {
                        Ok(body) => {
                            let content: &str = body["choices"][0]["message"]["content"]
                                .as_str()
                                .unwrap_or("false")
                                .trim();

                            let is_relevant: bool = content == "true";

                            #[cfg(debug_assertions)]
                            println!(
                                "Filter (using {}): '{}'\nResponse: '{}' -> {}",
                                current_model,
                                title,
                                content,
                                if is_relevant {
                                    "✅"
                                } else {
                                    "❌"
                                }
                            );
                            Ok(Some(is_relevant))
                        }
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            eprintln!(
                                "JSON parse error on attempt {} with model {}: {}",
                                attempt + 1,
                                current_model,
                                e
                            );
                            Ok(None)
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Request error on attempt {} with model {}: {}",
                        attempt + 1,
                        current_model,
                        e
                    );
                    Ok(None)
                }
            }
        Err(_) => {
            eprintln!("Timeout on attempt {} with model {}", attempt + 1, current_model);
            Ok(None)
        }
    }
}
