use std::process::Command;

fn main() {
    // Get current date/time
    let output = Command::new("date")
        .arg("+%Y-%m-%d %H:%M:%S UTC")
        .env("TZ", "UTC")
        .output()
        .unwrap();

    let build_date = String::from_utf8(output.stdout).unwrap().trim().to_string();

    println!("cargo:rustc-env=BUILD_DATE={}", build_date);
    println!("cargo:rerun-if-changed=build.rs");
}
