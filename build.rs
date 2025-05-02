use std::fs;
use std::process::Command;
use std::time::SystemTime;

fn main() {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    fs::write("build-timestamp.txt", timestamp.to_string()).expect("Failed to write timestamp");

    println!("cargo:rerun-if-changed=build-timestamp.txt");

    let output = Command::new("bash")
        .arg("sync-corpus.sh")
        .output()
        .expect("Failed to execute script");

    if !output.status.success() {
        panic!("Script failed with exit code: {}", output.status);
    }
}
