use std::process::Command;

fn main() {
    let Ok(output) = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
    else {
        return;
    };

    let git_short_hash = String::from_utf8(output.stdout).unwrap();

    println!("cargo:rustc-env=GIT_SHA_SHORT={}", git_short_hash.trim());
}
