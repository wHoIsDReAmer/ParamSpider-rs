use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::process;

use crate::cli::Cli;
use crate::constants::{COLOR_CYAN, COLOR_GREEN, COLOR_RESET, HARDCODED_EXTENSIONS};
use crate::http::fetch_url_content;
use crate::logging::{error, info};
use crate::url_clean::clean_urls;

pub fn run(cli: Cli) {
    if let Some(domain) = cli.domain.as_deref() {
        fetch_and_clean_urls(domain, cli.stream, cli.proxy.as_deref(), &cli.placeholder);
    }

    if let Some(list_path) = cli.list.as_deref() {
        let domains = read_domains_from_list(list_path);
        for domain in domains {
            fetch_and_clean_urls(&domain, cli.stream, cli.proxy.as_deref(), &cli.placeholder);
        }
    }
}

fn fetch_and_clean_urls(domain: &str, stream_output: bool, proxy: Option<&str>, placeholder: &str) {
    info(&format!(
        "Fetching URLs for {COLOR_CYAN}{domain}{COLOR_RESET}"
    ));
    let wayback_uri = format!(
        "https://web.archive.org/cdx/search/cdx?url={domain}/*&output=txt&collapse=urlkey&fl=original&page=/"
    );
    let response_text = fetch_url_content(&wayback_uri, proxy);
    let urls: Vec<String> = response_text
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    info(&format!(
        "Found {COLOR_GREEN}{}{COLOR_RESET} URLs for {COLOR_CYAN}{domain}{COLOR_RESET}",
        urls.len()
    ));

    let cleaned_urls = clean_urls(&urls, &HARDCODED_EXTENSIONS, placeholder);
    info(&format!(
        "Cleaning URLs for {COLOR_CYAN}{domain}{COLOR_RESET}"
    ));
    info(&format!(
        "Found {COLOR_GREEN}{}{COLOR_RESET} URLs after cleaning",
        cleaned_urls.len()
    ));
    info("Extracting URLs with parameters");

    let results_dir = "results";
    if let Err(err) = fs::create_dir_all(results_dir) {
        error(&format!("Failed to create results directory: {err}"));
        process::exit(1);
    }
    let result_file = format!("{results_dir}/{domain}.txt");
    let file = match File::create(&result_file) {
        Ok(file) => file,
        Err(err) => {
            error(&format!("Failed to create result file: {err}"));
            process::exit(1);
        }
    };
    let mut writer = std::io::BufWriter::new(file);
    for url in cleaned_urls {
        if url.contains('?') {
            let _ = writeln!(writer, "{url}");
            if stream_output {
                println!("{url}");
            }
        }
    }

    info(&format!(
        "Saved cleaned URLs to {COLOR_CYAN}{result_file}{COLOR_RESET}"
    ));
}

fn read_domains_from_list(path: &str) -> Vec<String> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            error(&format!("Failed to open list file: {err}"));
            process::exit(1);
        }
    };
    let reader = BufReader::new(file);
    let mut domains = HashSet::new();
    for line in reader.lines() {
        match line {
            Ok(raw) => {
                let cleaned = raw
                    .trim()
                    .to_lowercase()
                    .replace("https://", "")
                    .replace("http://", "");
                if !cleaned.is_empty() {
                    domains.insert(cleaned);
                }
            }
            Err(err) => {
                error(&format!("Failed to read list file: {err}"));
                process::exit(1);
            }
        }
    }
    domains.into_iter().collect()
}
