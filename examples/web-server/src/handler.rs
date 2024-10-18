use std::fs;
use crate::config::HandlerConfig;
use actix_web::{http::Method, web::Data, HttpRequest, HttpResponse};
use log::info;
use http_req::request;

// Implement your function's logic here
pub async fn index(req: HttpRequest, config: Data<HandlerConfig>) -> HttpResponse {
    //info!("{:#?}", req);
    if req.method() == Method::GET {
        let args: Vec<String> = std::env::args().collect();
        println!("args: {:?}", args);
        let storage_ip = std::env::var("STORAGE_IP").expect("Error: STORAGE_URL not found");
        println!("Value of STORAGE_IP: {}", storage_ip);

        println!("Downloading file");

        let file:String = args[2].parse().unwrap();
        let mut writer = Vec::new(); //container for body of a response
        let res = request::get("http://".to_owned()+&storage_ip+ &"/files/".to_owned()+&file, &mut writer).unwrap();
        println!("Status: {} {}", res.status_code(), res.reason());
        let len = writer.len();
        println!("Start transfer of {} at {}", len, chrono::offset::Utc::now());
        HttpResponse::Ok().body(writer)
    } else {
        HttpResponse::Ok().body(format!("Thanks {}!\n", config.name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{body::to_bytes, http, test::TestRequest, web::Bytes};

    fn config() -> Data<HandlerConfig> {
        Data::new(HandlerConfig::default())
    }

    #[actix_rt::test]
    async fn get() {
        let req = TestRequest::get().to_http_request();
        let resp = index(req, config()).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        assert_eq!(
            &Bytes::from(format!("Hello {}!\n", "world")),
            to_bytes(resp.into_body()).await.unwrap().as_ref()
        );
    }

    #[actix_rt::test]
    async fn post() {
        let req = TestRequest::post().to_http_request();
        let resp = index(req, config()).await;
        assert!(resp.status().is_success());
        assert_eq!(
            &Bytes::from(format!("Thanks {}!\n", "world")),
            to_bytes(resp.into_body()).await.unwrap().as_ref()
        );
    }
}
