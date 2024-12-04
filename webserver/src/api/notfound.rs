use actix_web::{HttpResponse, Responder};

pub async fn get() -> actix_web::Result<impl Responder> {
    Ok(HttpResponse::NotFound())
}
