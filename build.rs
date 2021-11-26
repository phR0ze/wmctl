use chrono::prelude::*;
use std::{fs, path::Path};

fn main() {
    // APP_BUILD_DATE
    let local: DateTime<Local> = Local::now();
    println!("cargo:rustc-env=APP_BUILD_DATE={}.{:0>2}.{:0>2}", local.year(), local.month(), local.day());

    // APP_GIT_COMMIT
    // ---------------------------------------------------------------------
    // Extract the last git commit SHA if in a git repo.
    // Target SHA is the second value in the last line of the HEAD logs file
    let mut git_hash = "NOT-FOUND".to_string();
    let head = Path::new(".git/logs/HEAD");
    if head.exists() {
        if let Ok(data) = fs::read_to_string(head) {
            if let Some(lastline) = data.lines().rev().next() {
                if let Some(hash) = lastline.split_ascii_whitespace().skip(1).next() {
                    git_hash = hash.to_string();
                }
            }
        }
    }
    println!("cargo:rustc-env=APP_GIT_COMMIT={}", git_hash);
}
