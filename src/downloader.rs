use anyhow::{Context, Result};
use colored::*;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::Write;
use std::path::Path;

pub async fn download_from_github(url: &str, target_dir: &Path) -> Result<String> {
    let parsed_url = url::Url::parse(url)?;

    if parsed_url.host_str() != Some("github.com") {
        return Err(anyhow::anyhow!("Not a GitHub URL"));
    }

    let segments: Vec<&str> = parsed_url
        .path()
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    // Require at least: owner/repo/blob/branch/file.ext
    if segments.len() < 5 {
        return Err(anyhow::anyhow!(
            "Invalid GitHub URL. Please provide a direct link to a file.\n\
             Example: https://github.com/user/repo/blob/main/agent.md"
        ));
    }

    // Check if it's a file URL (contains /blob/)
    if segments[2] != "blob" {
        return Err(anyhow::anyhow!(
            "Only direct file links are supported.\n\
             Please navigate to the specific agent file on GitHub and use that URL.\n\
             Example: https://github.com/user/repo/blob/main/agent.md"
        ));
    }

    let owner = segments[0];
    let repo = segments[1];
    let branch = segments[3];
    let file_path: Vec<&str> = segments[4..].to_vec();
    let full_path = file_path.join("/");
    let filename = file_path
        .last()
        .ok_or_else(|| anyhow::anyhow!("No filename in URL"))?
        .to_string();

    // Convert to raw content URL
    let raw_url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{}",
        owner, repo, branch, full_path
    );

    println!("  {} Downloading: {}", "→".cyan(), filename);

    let client = reqwest::Client::new();
    let response = client
        .get(&raw_url)
        .send()
        .await
        .with_context(|| format!("Failed to fetch {}", raw_url))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download file: HTTP {}\n\
             Make sure the file exists and the URL is correct.",
            response.status()
        ));
    }

    let total_size = response.content_length().unwrap_or(0);

    // Create progress bar
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("#>-"),
    );

    // Create target file path
    fs::create_dir_all(target_dir)?;
    let target_file = target_dir.join(&filename);
    let mut file = fs::File::create(&target_file)?;

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Failed to download chunk")?;
        file.write_all(&chunk)?;
        let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message("Download complete");

    Ok(filename)
}
