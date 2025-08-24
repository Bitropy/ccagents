use std::process::Command;

fn main() {
    // Try to get git describe output for detailed version
    let git_describe = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string());

    // Try to get git commit hash
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string());

    // Get build timestamp
    let build_timestamp = chrono::Utc::now().to_rfc3339();

    // Export the values as environment variables for use in the code
    if let Some(describe) = git_describe {
        println!("cargo:rustc-env=GIT_DESCRIBE={}", describe);
    }

    if let Some(hash) = git_hash {
        println!("cargo:rustc-env=GIT_HASH={}", hash);
    }

    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_timestamp);

    // Rebuild if git HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
}
