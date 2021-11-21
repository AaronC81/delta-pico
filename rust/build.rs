// Copied from: https://michael-f-bryan.github.io/rust-ffi-guide/cbindgen.html

extern crate cbindgen;

use std::{env, process::Command};
use std::path::PathBuf;
use cbindgen::{Config, Language};


fn main() {
    // Run cbindgen
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let output_file = target_dir()
        .join("include")
        .join("delta_pico_rust.h")
        .display()
        .to_string();

    let config = Config {
        namespace: Some(String::from("ffi")),
        language: Language::C,
        include_guard: Some("DELTA_PICO_RUST".into()),
        ..Default::default()
    };

    cbindgen::generate_with_config(&crate_dir, config)
      .unwrap()
      .write_to_file(&output_file);

    // Set GIT_VERSION environment variable
    let git_hash_output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(git_hash_output.stdout).unwrap();
    let git_modified = Command::new("git")
        .args(&["diff-index", "--quiet", "HEAD"])
        .status()
        .unwrap()
        .code().unwrap() != 0;

    let git_version = format!("{}{}", git_hash.trim(), if git_modified { "-modified" } else { "" });

    println!("cargo:rustc-env=GIT_VERSION={}", git_version);
}

/// Find the location of the `target/` directory. Note that this may be 
/// overridden by `cmake`, so we also need to check the `CARGO_TARGET_DIR` 
/// variable.
fn target_dir() -> PathBuf {
    if let Ok(target) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(target)
    } else {
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("target")
    }
}
