use crate::constants::{COLOR_RESET, COLOR_YELLOW};

pub fn info(message: &str) {
    println!("{COLOR_YELLOW}[INFO]{COLOR_RESET} {message}");
}

pub fn warn(message: &str) {
    println!("{COLOR_YELLOW}[WARN]{COLOR_RESET} {message}");
}

pub fn error(message: &str) {
    eprintln!("{COLOR_YELLOW}[ERROR]{COLOR_RESET} {message}");
}
