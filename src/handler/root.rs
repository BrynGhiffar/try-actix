use std::time::{ SystemTime, UNIX_EPOCH };
use actix_web::Responder;

pub async fn healthcheck() -> impl Responder {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)
                                .map(|t| t.as_secs())
                                .unwrap();
    format!("Server is ok: {timestamp}")
}