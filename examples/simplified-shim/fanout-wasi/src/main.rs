use wasmedge_http_req::request;
use std::env;

// Use a custom Result type alias
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn download_file_content(storage_ip: &str, file_name: &str) -> Result<String> {
    let url = format!("http://{}/files/{}", storage_ip, file_name);
    println!("downloading {}", url);
    let mut writer = Vec::new(); //container for body of a response
    let _ = request::get(url, &mut writer).unwrap();
    let response_string = unsafe {String::from_utf8_unchecked(writer)};
    Ok(format!("result {:?}",response_string))
}

async fn fanout_task(task_id: usize, content: String) {
    println!("Starting task {}", task_id);
    let content_task = format!("task={} start {}",task_id, content);
    let len = content_task.len() as i32;
    let ptr_i32 = content_task.as_ptr() as i32;
    println!("start transfer at {}",chrono::offset::Utc::now());
    let _ = unsafe{ cwasi_export::func_connect(ptr_i32,len)};
    println!("Finished task {}", task_id);
}

fn fetch_env_variables() -> (String, String, String,usize) {
    //let storage_ip = env::var("STORAGE_IP").expect("Error: STORAGE_IP not found");
    //let funcb_url = env::var("FUNCB_URL").expect("Error: FUNCB_URL not found");
    let funcb_url= "funcb".to_string();
    let storage_ip = env::var("STORAGE_IP").expect("Error: STORAGE_IP not found");
    let file_name = env::args().nth(2).expect("Error: Missing file argument");

    let num_tasks: usize = env::var("NUM_TASKS").expect("Error: Missing num tasks")
        .parse().expect("NUM_TASKS must be a valid number");

    (storage_ip, funcb_url, file_name, num_tasks)
}

// Main function that orchestrates the fanout logic
fn main() -> Result<()> {
    // Fetch environment variables, including the number of tasks
    let (storage_ip, _funcb_url, file_name, num_tasks) = fetch_env_variables();

    // Create a Tokio runtime to handle async execution
    let tasks = async {
        // Download the file content once
        let file_content = download_file_content(&storage_ip, &file_name).await.unwrap();

        // Create a vector to store the tasks
        let mut task_handles = Vec::new();

        // Spawn the number of tasks defined in the environment
        for i in 0..num_tasks {
            let content_clone = file_content.clone();
            task_handles.push(tokio::spawn(fanout_task(i + 1, content_clone)));
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

pub mod cwasi_export {
    #[link(wasm_import_module = "cwasi_export")]
    extern "C" {
        pub fn func_connect(ptr: i32, len: i32) -> i32;
    }
}