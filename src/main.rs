use std::process::{Command, Output};
use std::thread;
use std::time::Duration;
use chrono::{DateTime, Utc};
use std::str;

fn func_a() -> Option<String> {
    println!("Starting func_a...");

    // Pull the latest image for func_a
    let pull_a_status = Command::new("crictl")
        .args(&["pull", "docker.io/keniack/func_a:latest"])
        .status()
        .expect("Failed to pull func_a image");

    if !pull_a_status.success() {
        println!("Failed to pull func_a image.");
        return None;
    }

    println!("Successfully pulled func_a image.");

    // Run the func_a container using ctr and capture the output
    let run_a_output: Output = Command::new("ctr")
        .args(&[
            "-n", "k8s.io", "run", "--rm", "--runtime=io.containerd.cwasi.v1",
            "--net-host=true",
            "--env", "STORAGE_IP=127.0.0.1:8888",
            "--env", "REDIS_IP=192.168.0.38",
            "--env", "FUNCTIONS_NUM=1",
            "docker.io/keniack/func_a:latest", &format!("{}", rand::random::<u16>()),
            "/func_a.wasm", "func_b.wasm", "hello.txt",
        ])
        .output()
        .expect("Failed to run func_a");

    if run_a_output.status.success() {
        println!("func_a executed successfully.");
    } else {
        println!("func_a failed to execute.");
        return None;
    }

    // Convert output to a string and search for the relevant line
    let output_str = str::from_utf8(&run_a_output.stdout).expect("Failed to read output");
    if let Some(start_transfer_line) = output_str.lines().find(|line| line.contains("start transfer at")) {
        println!("Found func_a timestamp: {}", start_transfer_line);
        return Some(start_transfer_line.to_string());
    }

    None
}

fn func_b() -> Option<String> {
    println!("Starting func_b...");

    // Pull the latest image for func_b
    let pull_b_status = Command::new("crictl")
        .args(&["pull", "docker.io/keniack/func_b:latest"])
        .status()
        .expect("Failed to pull func_b image");

    if !pull_b_status.success() {
        println!("Failed to pull func_b image.");
        return None;
    }

    println!("Successfully pulled func_b image.");

    thread::sleep(Duration::from_secs(1));

    // Run the func_b container using ctr and capture the output
    let run_b_output: Output = Command::new("ctr")
        .args(&[
            "-n", "k8s.io", "run", "--rm", "--runtime=io.containerd.cwasi.v1",
            "--annotation", "cwasi.secondary.function=true",
            "--net-host=true",
            "docker.io/keniack/func_b:latest", &format!("{}", rand::random::<u16>()),
            "/func_b.wasm",
        ])
        .output()
        .expect("Failed to run func_b");

    if run_b_output.status.success() {
        println!("func_b executed successfully.");
    } else {
        println!("func_b failed to execute.");
        return None;
    }

    // Convert output to a string and search for the relevant line
    let output_str = str::from_utf8(&run_b_output.stdout).expect("Failed to read output");
    if let Some(args_read_line) = output_str.lines().find(|line| line.contains("Args read at")) {
        println!("Found func_b timestamp: {}", args_read_line);
        return Some(args_read_line.to_string());
    }

    None
}

fn extract_timestamp(line: &str) -> Option<DateTime<Utc>> {
    // Try to extract a DateTime from the line using a regex or simple parsing
    let timestamp_str = line.split_whitespace().nth(3)?;
    DateTime::parse_from_rfc3339(timestamp_str).ok().map(|dt| dt.with_timezone(&Utc))
}

fn main() {
    // Run func_a in a separate thread and capture the return value
    let func_a_thread = thread::spawn(|| {
        return func_a();
    });

    // Run func_b in a separate thread and capture the return value
    let func_b_thread = thread::spawn(|| {
        return func_b();
    });

    // Get the results from both threads
    let func_a_line = func_a_thread.join().expect("func_a thread panicked").unwrap();
    let func_b_line = func_b_thread.join().expect("func_b thread panicked").unwrap();

    let func_a_timestamp = extract_timestamp(&func_a_line).unwrap();
    let func_b_timestamp = extract_timestamp(&func_b_line).unwrap();
    let duration = func_b_timestamp.signed_duration_since(func_a_timestamp);
    println!("Duration between func_a and func_b: {} milliseconds", duration.num_milliseconds());

    println!("Both func_a and func_b have completed.");
}