mod app;
mod banner;
mod cli;
mod constants;
mod http;
mod logging;
mod url_clean;

use clap::Parser;

use crate::cli::Cli;
use crate::constants::{COLOR_RESET, COLOR_YELLOW};

fn main() {
    println!("{COLOR_YELLOW}{}{COLOR_RESET}", banner::BANNER);

    let cli = Cli::parse();
    app::run(cli);
}
