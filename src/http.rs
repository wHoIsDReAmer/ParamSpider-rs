use rand::seq::IndexedRandom;
use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use std::process;
use std::thread::sleep;
use std::time::Duration;

use crate::constants::{MAX_RETRIES, RETRY_DELAY_SECS, USER_AGENTS};
use crate::logging::{error, warn};

fn normalize_proxy(proxy: &str) -> String {
    if proxy.contains("://") {
        proxy.to_string()
    } else {
        format!("http://{proxy}")
    }
}

fn build_client(proxy: Option<&str>) -> Result<Client, reqwest::Error> {
    let mut builder = Client::builder();
    if let Some(proxy_value) = proxy {
        let proxy_url = normalize_proxy(proxy_value);
        builder = builder.proxy(reqwest::Proxy::all(&proxy_url)?);
    }
    builder.build()
}

pub fn fetch_url_content(url: &str, proxy: Option<&str>) -> String {
    let client = match build_client(proxy) {
        Ok(client) => client,
        Err(err) => {
            error(&format!("Failed to configure HTTP client: {err}"));
            process::exit(1);
        }
    };

    let mut rng = rand::rng();
    for _ in 0..MAX_RETRIES {
        let user_agent = USER_AGENTS
            .choose(&mut rng)
            .copied()
            .unwrap_or("Mozilla/5.0");
        let response = client.get(url).header(USER_AGENT, user_agent).send();
        match response {
            Ok(resp) => match resp.error_for_status() {
                Ok(ok) => match ok.text() {
                    Ok(text) => return text,
                    Err(_err) => {
                        warn(&format!(
                            "Error fetching URL {url}. Retrying in 5 seconds..."
                        ));
                        sleep(Duration::from_secs(RETRY_DELAY_SECS));
                    }
                },
                Err(_err) => {
                    warn(&format!(
                        "Error fetching URL {url}. Retrying in 5 seconds..."
                    ));
                    sleep(Duration::from_secs(RETRY_DELAY_SECS));
                }
            },
            Err(_err) => {
                warn(&format!(
                    "Error fetching URL {url}. Retrying in 5 seconds..."
                ));
                sleep(Duration::from_secs(RETRY_DELAY_SECS));
            }
        }
    }

    error(&format!(
        "Failed to fetch URL {url} after {MAX_RETRIES} retries."
    ));
    process::exit(1);
}
