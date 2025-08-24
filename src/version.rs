use colored::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

pub fn print_version_info() {
    println!("{} {}", PKG_NAME.cyan().bold(), VERSION.green());

    if let Some(git_describe) = option_env!("GIT_DESCRIBE") {
        println!("Git version: {}", git_describe.dimmed());
    } else if let Some(git_hash) = option_env!("GIT_HASH") {
        println!("Git commit: {}", git_hash.dimmed());
    }

    if let Some(build_time) = option_env!("BUILD_TIMESTAMP") {
        println!("Built: {}", build_time.dimmed());
    }

    println!("Author: {}", AUTHORS.dimmed());
}

#[allow(dead_code)]
pub fn get_version_string() -> String {
    if let Some(git_describe) = option_env!("GIT_DESCRIBE") {
        format!("{} ({})", VERSION, git_describe)
    } else {
        VERSION.to_string()
    }
}
