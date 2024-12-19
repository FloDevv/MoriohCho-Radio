use crate::config::types::FilterConfig;

//filter who remove all atricle who have banned word on title or description
pub async fn bannedfilter(
    title: &str,
    description: &str,
    filter_config: &FilterConfig
) -> Result<bool, Box<dyn std::error::Error>> {
    let title_lower: String = title.to_lowercase();
    let description_lower: String = description.to_lowercase();
    let mut is_relevant: bool = true;

    for banned in &filter_config.banned {
        let banned_lower: String = banned.to_lowercase();
        let variants: Vec<String> = if banned_lower.ends_with('s') {
            vec![banned_lower.clone(), banned_lower.trim_end_matches('s').to_string()]
        } else {
            vec![banned_lower.clone(), format!("{}s", banned_lower)]
        };

        for variant in variants {
            if title_lower.contains(&variant) || description_lower.contains(&variant) {
                is_relevant = false;
                break;
            }
        }

        if !is_relevant {
            break;
        }
    }
    Ok(is_relevant)
}
