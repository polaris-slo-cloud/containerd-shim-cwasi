use std::env;

// Use a custom Result type alias
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn fanout_task(task_id: usize, content: String) {
    let content_task = format!("task={} start {}",task_id, content);
    let len = content_task.len() as i32;
    let ptr_i32 = content_task.as_ptr() as i32;
    println!("Starting task {}", task_id);
    println!("start transfer of {} at {}",len, chrono::offset::Utc::now());
    let _ = unsafe{ cwasi_export::func_connect(ptr_i32,len)};
    println!("Finished task {}", task_id);
}

fn fetch_env_variables() -> (String, Option<String>, String,usize) {
    //let storage_ip = env::var("STORAGE_IP").expect("Error: STORAGE_IP not found");
    let funcb_url: Option<String> = env::var("FUNCB_URL").ok();
    let storage_ip = env::var("STORAGE_IP").expect("Error: STORAGE_IP not found");
    let file_name = env::args().nth(2).expect("Error: Missing file argument");

    let num_tasks: usize = env::var("NUM_TASKS").expect("Error: Missing num tasks")
        .parse().expect("NUM_TASKS must be a valid number");

    (storage_ip, funcb_url, file_name, num_tasks)
}

fn main() -> Result<()> {
    println!("Fetching env variables");
    let args: Vec<String> = std::env::args().collect();
    let (_storage_ip, _funcb_url, _file_name, num_tasks) = fetch_env_variables();

    let file_content = args[3].clone();

    let tasks = async {
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