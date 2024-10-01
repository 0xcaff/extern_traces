use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .expect("failed to get git short commit sha");

    let git_short_hash = String::from_utf8(output.stdout).unwrap();

    println!("cargo:rustc-env=GIT_SHA_SHORT={}", git_short_hash.trim());
    println!("cargo:rustc-link-lib=framework=Metal");
}
