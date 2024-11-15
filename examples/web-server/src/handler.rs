
use crate::config::HandlerConfig;
use actix_web::{http::Method, web, web::Data, HttpMessage, HttpRequest, HttpResponse};
use actix_web::web::Bytes;
use http_req::request;

// Implement your function's logic here
pub async fn index(req: HttpRequest, body: Bytes) -> HttpResponse {
    //info!("{:#?}", req);
    if req.method() == Method::GET {
        let args: Vec<String> = std::env::args().collect();
        println!("args: {:?}", args);
        let storage_ip = std::env::var("STORAGE_IP").expect("Error: STORAGE_URL not found");
        println!("Value of STORAGE_IP: {}", storage_ip);

        println!("Downloading file");
        let req_id = req.headers()
            .get("Req-Header")
            .and_then(|header_value| header_value.to_str().ok())
            .unwrap_or("0");

        let file:String = args[2].parse().unwrap();
        let mut writer = Vec::new(); //container for body of a response
        let res = request::get("http://".to_owned()+&storage_ip+ &"/files/".to_owned()+&file, &mut writer).unwrap();
        println!("Status: {} {}", res.status_code(), res.reason());
        let len = writer.len();
        println!("Start transfer of {} at {} for {}", len, chrono::offset::Utc::now(),req_id);
        HttpResponse::Ok().body(writer)
    } else if req.method() == Method::POST{
        let mut response = format!("Received {} at {:?}", body.len(), chrono::offset::Utc::now());
        let body_string = String::from_utf8(body.to_vec()).unwrap();
        response = format!("{} \nAfter serialization at {:?} ",response, chrono::offset::Utc::now());
        HttpResponse::Ok().body(response)
    } else {
        HttpResponse::Ok().body(format!("Thanks!!\n"))
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
