use std::process::{Command, Output};
use std::thread;
use std::time::Duration;
use std::str;
use chrono::{DateTime, Utc};

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
            "/func_a.wasm", "func_b.wasm", "file_1M.txt",
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
        return Some(start_transfer_line.split('"').nth(1)?.to_string());
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
    if let Some(args_read_line) = output_str.lines().find(|line| line.contains("FnB Shim Finished. Result from moduleB")) {
        println!("Found func_b timestamp: {}", args_read_line);
        return Some(args_read_line.to_string().split_whitespace().last().unwrap().to_string());
    }

    None
}

pub fn epoch_todate(now_nanos:i64) -> DateTime<Utc> {
    // Convert the nanosecond timestamp back to seconds and nanoseconds
    let timestamp_seconds = now_nanos / 1_000_000_000;
    let timestamp_nanoseconds = (now_nanos % 1_000_000_000) as u32;

    let datetime_utc = DateTime::<Utc>::from_timestamp(timestamp_seconds, timestamp_nanoseconds).unwrap();
    return datetime_utc;
}

fn main() {
    // Run func_a in a separate thread and capture the return value
    let func_a_thread = thread::spawn(|| {
        func_a()
    });

    // Run func_b in a separate thread and capture the return value
    let func_b_thread = thread::spawn(|| {
        func_b()
    });

    // Get the results from both threads
    let func_a_result = func_a_thread.join().expect("func_a thread panicked").unwrap();
    let func_b_result = func_b_thread.join().expect("func_b thread panicked").unwrap();

    let start_date = func_a_result.parse::<DateTime<Utc>>().unwrap();
    let end_date = epoch_todate(func_b_result.parse::<i64>().unwrap());
    println!("end date {}",end_date);
    println!("start date {}",start_date);

    let duration = end_date - start_date;
    println!("Duration {}",duration);

    println!("Both func_a and func_b have completed.");
}