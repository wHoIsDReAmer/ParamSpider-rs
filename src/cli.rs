use clap::{ArgGroup, Parser};

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
pub struct Cli {
    #[arg(
        short = 'd',
        long = "domain",
        help = "Domain name to fetch related URLs for."
    )]
    pub domain: Option<String>,

    #[arg(
        short = 'l',
        long = "list",
        help = "File containing a list of domain names."
    )]
    pub list: Option<String>,

    #[arg(short = 's', long = "stream", help = "Stream URLs on the terminal.")]
    pub stream: bool,

    #[arg(long = "proxy", help = "Set the proxy address for web requests.")]
    pub proxy: Option<String>,

    #[arg(
        short = 'p',
        long = "placeholder",
        help = "placeholder for parameter values",
        default_value = "FUZZ"
    )]
    pub placeholder: String,
}
