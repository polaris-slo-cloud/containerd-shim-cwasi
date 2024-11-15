use reqwest::blocking::{Client, Response}; // Use the blocking client
use std::{env, error::Error};
use reqwest::header::{HeaderMap, HeaderValue};

// Use a custom Result type alias
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn http_client(url: &str, task_id: usize) -> Result<String> {
    let client = Client::new();

    // Create headers
    let mut headers = HeaderMap::new();
    let req_id = task_id;
    headers.insert("Req-Header", HeaderValue::from_str(req_id.to_string().as_str())?);

    // Make the request with headers
    let response: Response = client.get(url)
        .headers(headers)
        .send()?; // Blocking send, no await

    println!(
        "Received chunk of {:?} at {:?} for {:?}",
        response.content_length().unwrap(),
        chrono::offset::Utc::now(),
        task_id
    );
    let body = response.text()?;
    println!("After serialization at {:?} for {:?}", chrono::offset::Utc::now(), task_id);

    Ok(body)
}

fn fanout_task(task_id: usize, func_b_url: String) {
    println!("Starting task {}", task_id);
    println!("Connect to {:?}", func_b_url);
    let _response_string = http_client(func_b_url.as_str(), task_id).unwrap();
    println!("Finished task {}", task_id);
}

fn fetch_env_variables() -> (String, usize) {
    let funcb_url: String = env::var("FUNCB_URL").expect("Error: FUNCB_URL not found");

    let num_tasks: usize = env::var("NUM_TASKS")
        .expect("Error: Missing num tasks")
        .parse()
        .expect("NUM_TASKS must be a valid number");

    (funcb_url, num_tasks)
}

// Main function that orchestrates the fanout logic
fn main() -> Result<()> {
    // Fetch environment variables, including the number of tasks
    let (funcb_url, num_tasks) = fetch_env_variables();

    // Execute tasks synchronously in sequence
    for i in 0..num_tasks {
        fanout_task(i + 1, funcb_url.clone());
    }

    println!("All tasks completed.");
    Ok(())
}