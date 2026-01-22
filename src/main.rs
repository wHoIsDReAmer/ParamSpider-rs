use clap::{ArgGroup, Parser};
use rand::seq::IndexedRandom;
use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::process;
use std::thread::sleep;
use std::time::Duration;
use url::{Url, form_urlencoded};

const HARDCODED_EXTENSIONS: [&str; 17] = [
    ".jpg", ".jpeg", ".png", ".gif", ".pdf", ".svg", ".json", ".css", ".js", ".webp", ".woff",
    ".woff2", ".eot", ".ttf", ".otf", ".mp4", ".txt",
];

const USER_AGENTS: [&str; 15] = [
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36",
    "Mozilla/5.0 (Windows NT 6.1; WOW64; rv:54.0) Gecko/20100101 Firefox/54.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:54.0) Gecko/20100101 Firefox/54.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_6) AppleWebKit/603.3.8 (KHTML, like Gecko) Version/10.1.2 Safari/603.3.8",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.82 Safari/537.36 Edg/89.0.774.45",
    "Mozilla/5.0 (Windows NT 10.0; WOW64; Trident/7.0; AS; rv:11.0) like Gecko",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.96 Safari/537.36 Edge/16.16299",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36 OPR/45.0.2552.898",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36 Vivaldi/1.8.770.50",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:54.0) Gecko/20100101 Firefox/54.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36 Edge/15.15063",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36 Edge/15.15063",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.81 Safari/537.36",
];

const MAX_RETRIES: usize = 3;
const RETRY_DELAY_SECS: u64 = 5;

const COLOR_YELLOW: &str = "\x1b[33m";
const COLOR_CYAN: &str = "\x1b[36m";
const COLOR_GREEN: &str = "\x1b[32m";
const COLOR_RESET: &str = "\x1b[0m";

#[derive(Parser)]
#[command(
    name = "paramspider",
    about = "Mining URLs from dark corners of Web Archives"
)]
#[command(
    group = ArgGroup::new("input")
        .required(true)
        .multiple(false)
        .args(["domain", "list"])
)]
struct Cli {
    #[arg(
        short = 'd',
        long = "domain",
        help = "Domain name to fetch related URLs for."
    )]
    domain: Option<String>,

    #[arg(
        short = 'l',
        long = "list",
        help = "File containing a list of domain names."
    )]
    list: Option<String>,

    #[arg(short = 's', long = "stream", help = "Stream URLs on the terminal.")]
    stream: bool,

    #[arg(long = "proxy", help = "Set the proxy address for web requests.")]
    proxy: Option<String>,

    #[arg(
        short = 'p',
        long = "placeholder",
        help = "placeholder for parameter values",
        default_value = "FUZZ"
    )]
    placeholder: String,
}

fn info(message: &str) {
    println!("{COLOR_YELLOW}[INFO]{COLOR_RESET} {message}");
}

fn warn(message: &str) {
    println!("{COLOR_YELLOW}[WARN]{COLOR_RESET} {message}");
}

fn error(message: &str) {
    eprintln!("{COLOR_YELLOW}[ERROR]{COLOR_RESET} {message}");
}

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

fn clean_urls(urls: &[String], extensions: &[&str], placeholder: &str) -> Vec<String> {
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

fn fetch_url_content(url: &str, proxy: Option<&str>) -> String {
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
                    Err(err) => {
                        warn(&format!(
                            "Error fetching URL {url}. Retrying in 5 seconds..."
                        ));
                        let _ = err;
                        sleep(Duration::from_secs(RETRY_DELAY_SECS));
                    }
                },
                Err(err) => {
                    warn(&format!(
                        "Error fetching URL {url}. Retrying in 5 seconds..."
                    ));
                    let _ = err;
                    sleep(Duration::from_secs(RETRY_DELAY_SECS));
                }
            },
            Err(err) => {
                warn(&format!(
                    "Error fetching URL {url}. Retrying in 5 seconds..."
                ));
                let _ = err;
                sleep(Duration::from_secs(RETRY_DELAY_SECS));
            }
        }
    }

    error(&format!(
        "Failed to fetch URL {url} after {MAX_RETRIES} retries."
    ));
    process::exit(1);
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

fn main() {
    let log_text = r#"

                                      _    __
   ___  ___ ________ ___ _  ___ ___  (_)__/ /__ ____
  / _ \/ _ `/ __/ _ `/  ' \(_-</ _ \/ / _  / -_) __/
 / .__/\_,_/_/  \_,_/_/_/_/___/ .__/_/\_,_/\__/_/
/_/                          /_/

                              with <3 by @0xasm0d3us
    "#;
    println!("{COLOR_YELLOW}{log_text}{COLOR_RESET}");

    let cli = Cli::parse();

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
