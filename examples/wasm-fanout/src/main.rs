use wasmedge_http_req::request;
use tokio::task;
use std::env;

// Use a custom Result type alias
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn http_client(url:String) -> Vec<u8> {
    let mut writer = Vec::new(); //container for body of a response
    let _ = request::get(url, &mut writer).unwrap();
    return writer;
}


async fn http_client_non_blocking(url: String) -> Vec<u8> {
    // Spawn the blocking request directly in a task
    tokio::spawn(async move {
        let mut writer = Vec::new();
        request::get(&url, &mut writer).unwrap();
        //println!("Received chunk of size: {} at {:?}", writer.len(), chrono::offset::Utc::now());
        //let body_string = String::from_utf8(writer.to_vec()).unwrap();
        //println!("After serialization at {:?}", chrono::offset::Utc::now());
        writer
    })
        .await
        .unwrap() // Await the spawned task and unwrap the result
}

async fn fanout_task(task_id: usize, func_b_url: String) {
    println!("Starting task {}", task_id);
    println!("Connect to {:?}",func_b_url.clone());
    let mut writer = http_client_non_blocking(func_b_url).await;
    println!("Received chunk of size: {} at {:?}", writer.len(), chrono::offset::Utc::now());
    let body_string = String::from_utf8(writer.to_vec()).unwrap();
    println!("After serialization at {:?}", chrono::offset::Utc::now());

    println!("Finished task {}", task_id);
}

fn fetch_env_variables() -> ( String,usize) {
    //let storage_ip = env::var("STORAGE_IP").expect("Error: STORAGE_IP not found");
   // let funcb_url: String = env::var("FUNCB_URL").expect("Error: FUNCB not found");
    //let num_tasks: usize = env::var("NUM_TASKS").expect("Error: Missing num tasks")
    //    .parse().expect("NUM_TASKS must be a valid number");
    let funcb_url ="http://localhost:8080".to_string();
    let num_tasks:usize=20;
    (funcb_url, num_tasks)
}

// Main function that orchestrates the fanout logic
fn main() -> Result<()> {
    // Fetch environment variables, including the number of tasks
    let (funcb_url, num_tasks) = fetch_env_variables();

    // Create a Tokio runtime to handle async execution
    let tasks = async {

        // Create a vector to store the tasks
        let mut task_handles = Vec::new();

        // Spawn the number of tasks defined in the environment
        for i in 0..num_tasks {
            task_handles.push(tokio::spawn(fanout_task(i + 1, funcb_url.clone())));
        }

        // Wait for all tasks to complete
        for handle in task_handles {
            handle.await.unwrap();
        }

        println!("All tasks completed.");
    };

    // Execute the tasks using Tokio's current-thread runtime
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(tasks);
    Ok(())
}
