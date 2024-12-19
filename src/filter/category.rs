use crate::config::types::FilterConfig;

//filter who remove only from ai filter when categories are in title or description
pub async fn categoryfilter(
    title: &str,
    description: &str,
    filter_config: &FilterConfig
) -> Result<bool, Box<dyn std::error::Error>> {
    let title_lower: String = title.to_lowercase();
    let description_lower: String = description.to_lowercase();
    let mut is_relevant: bool = false;

    for category in &filter_config.categories {
        let category_lower: String = category.to_lowercase();
        let variants: Vec<String> = if category_lower.ends_with('s') {
            vec![category_lower.clone(), category_lower.trim_end_matches('s').to_string()]
        } else {
            vec![category_lower.clone(), format!("{}s", category_lower)]
        };

        for variant in variants {
            if title_lower.contains(&variant) || description_lower.contains(&variant) {
                is_relevant = true;
                break;
            }
        }

        if is_relevant {
            break;
        }
    }
    Ok(is_relevant)
}
