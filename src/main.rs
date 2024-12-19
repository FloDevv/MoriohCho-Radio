use std::error::Error;
use ai::filter::aifilter;
use ai::resume::ai_resume;
use config::types::{ Config, FilterConfig };
use fetch::{ news, weather };
use types::WeatherResponse;
use std::io::{ self, Write };
use filter::banned::bannedfilter;
use filter::category::categoryfilter;
pub mod types;
mod ai;
mod fetch;
mod config;
mod filter;
use futures::stream::{ self, StreamExt };
use reqwest::Client;
use indicatif::{ ProgressBar, ProgressStyle };

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize shared resources
    let client: Client = reqwest::Client::new();
    let config: Config = config::load_config()?;

    // Create progress style once
    let progress_style: ProgressStyle = ProgressStyle::default_bar()
        .template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)"
        )
        .unwrap();

    // Fetch weather
    println!("Fetching weather for {}...", config.city);
    let weather: WeatherResponse = weather::fetch_weather(&config.city).await?;

    // Fetch and filter articles
    let news_sources: Vec<&str> = config.news_sources
        .iter()
        .map(|source| source.url.as_str())
        .collect();

    let articles: Vec<types::Article> = news::fetch_news(&news_sources).await?;

    let mut filtered_articles: Vec<&types::Article> = Vec::new();

    for article in &articles {
        let banned: bool = bannedfilter(
            &article.title,
            &article.description,
            &config.filter
        ).await?;
        if !banned {
            continue;
        }

        let category_match: bool = categoryfilter(
            &article.title,
            &article.description,
            &config.filter
        ).await?;
        if category_match {
            filtered_articles.push(article);
            continue;
        }

        filtered_articles.push(article);
    }

    println!("Filtering with AI...");
    let filter_pb: ProgressBar = ProgressBar::new(filtered_articles.len() as u64).with_style(
        progress_style.clone()
    );

    // Filtrage AI uniquement sur les articles restants
    let ai_filtered_articles: Vec<&types::Article> = stream
        ::iter(filtered_articles.iter())
        .map(|article: &&types::Article| {
            let client: &Client = &client;
            let filter_pb: &ProgressBar = &filter_pb;
            let filter_clone: FilterConfig = config.filter.clone();
            let config_clone: Config = config.clone();
            async move {
                let is_relevant: bool = aifilter(
                    &article.title,
                    &article.description,
                    &config_clone,
                    &filter_clone,
                    client
                ).await.unwrap_or(false);
                filter_pb.inc(1);
                if is_relevant {
                    Some(*article)
                } else {
                    None
                }
            }
        })
        .buffer_unordered(1)
        .filter_map(|x: Option<&types::Article>| async move { x })
        .collect().await;

    filter_pb.finish_with_message("AI filtering done");

    // Fetch content for filtered articles
    println!("Fetching article content...");
    let fetch_pb: ProgressBar = ProgressBar::new(ai_filtered_articles.len() as u64).with_style(
        progress_style
    );

    let articles_with_content: Vec<types::Article> = stream
        ::iter(ai_filtered_articles)
        .map(|article: &types::Article| {
            let client: &Client = &client;
            let fetch_pb: &ProgressBar = &fetch_pb;
            async move {
                if !article.source.is_empty() {
                    let result: Result<types::Article, Box<dyn Error>> = news::fetch_article(
                        &article.title,
                        &article.source,
                        &article.description,
                        client
                    ).await;
                    fetch_pb.inc(1);
                    result.ok()
                } else {
                    fetch_pb.inc(1);
                    None
                }
            }
        })
        .buffer_unordered(25)
        .filter_map(|x: Option<types::Article>| async move { x })
        .collect().await;

    fetch_pb.finish_with_message("Content fetched");

    // Generate summary
    let articles_text: String = articles_with_content
        .iter()
        .map(|a: &types::Article|
            format!(
                "Title: {}\nDescription: {}\nContent: {}\n---\n",
                a.title,
                a.description,
                a.content
            )
        )
        .collect::<String>();

    println!("Generating summary...");
    let ai_summary: String = ai_resume(&weather, &articles_text, &client, &config).await?;
    println!("\nSummary:\n{}", ai_summary);
    let _ = io::stdout().flush();
    let mut buffer: String = String::new();
    let _ = io::stdin().read_line(&mut buffer);
    Ok(())
}
