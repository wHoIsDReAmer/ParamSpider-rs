use std::collections::HashSet;
use url::{Url, form_urlencoded};

fn has_extension(url: &str, extensions: &[&str]) -> bool {
    let Ok(parsed) = Url::parse(url) else {
        return false;
    };
    let path = parsed.path();
    let file_name = path.rsplit('/').next().unwrap_or("");
    let ext = match file_name.rfind('.') {
        Some(idx) if idx > 0 => file_name[idx..].to_lowercase(),
        _ => String::new(),
    };
    extensions.iter().any(|e| *e == ext)
}

fn clean_url(url: &str) -> String {
    let Ok(mut parsed) = Url::parse(url) else {
        return url.to_string();
    };
    if (parsed.scheme() == "http" && parsed.port() == Some(80))
        || (parsed.scheme() == "https" && parsed.port() == Some(443))
    {
        let _ = parsed.set_port(None);
    }
    parsed.to_string()
}

pub fn clean_urls(urls: &[String], extensions: &[&str], placeholder: &str) -> Vec<String> {
    let mut cleaned = HashSet::new();
    for url in urls {
        let cleaned_url = clean_url(url);
        if has_extension(&cleaned_url, extensions) {
            continue;
        }
        let Ok(mut parsed) = Url::parse(&cleaned_url) else {
            cleaned.insert(cleaned_url);
            continue;
        };
        let mut seen = HashSet::new();
        let mut ordered_keys = Vec::new();
        for (key, _) in parsed.query_pairs() {
            let key_str = key.to_string();
            if seen.insert(key_str.clone()) {
                ordered_keys.push(key_str);
            }
        }
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        for key in ordered_keys {
            serializer.append_pair(&key, placeholder);
        }
        let cleaned_query = serializer.finish();
        if cleaned_query.is_empty() {
            parsed.set_query(None);
        } else {
            parsed.set_query(Some(&cleaned_query));
        }
        cleaned.insert(parsed.to_string());
    }
    cleaned.into_iter().collect()
}
