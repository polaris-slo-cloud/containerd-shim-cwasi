use tonic::{transport::Server, Request, Response, Status};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub mod file_service {
    tonic::include_proto!("proto/fileservice"); // The string specified here must match the package name in the .proto file
}

use file_service::file_service_server::{FileService, FileServiceServer};
use file_service::{FileRequest, FileResponse};

#[derive(Debug, Default)]
pub struct MyFileService {}

#[tonic::async_trait]
impl FileService for MyFileService {
    async fn get_file(
        &self,
        request: Request<FileRequest>,
    ) -> Result<Response<FileResponse>, Status> {
        let filename = request.into_inner().filename;

        // Read the file asynchronously
        let mut file = match File::open(filename).await {
            Ok(file) => file,
            Err(_) => return Err(Status::not_found("File not found")),
        };

        let mut content = Vec::new();
        if let Err(_) = file.read_to_end(&mut content).await {
            return Err(Status::internal("Failed to read file"));
        }

        // Return the file content
        let response = FileResponse { content };
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let file_service = MyFileService::default();

    println!("FileService Server listening on {}", addr);

    Server::builder()
        .add_service(FileServiceServer::new(file_service))
        .serve(addr)
        .await?;

    Ok(())
}