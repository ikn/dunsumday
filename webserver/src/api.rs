use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse};
use actix_web::dev::HttpServiceFactory;
use dunsumday::config::Config;
use crate::configrefs;

mod item;
pub mod notfound;

pub const GET_ITEMS: &str = "get items";
pub const CREATE_ITEM: &str = "create item";

pub fn service<C>(cfg: &C) -> impl HttpServiceFactory
where
    C: Config + ?Sized,
{
    web::scope(cfg.get_ref(&configrefs::SERVER_API_PATH))
        .service(web::resource("/item").name(GET_ITEMS).get(item::list))
        .service(web::resource("/item").name(CREATE_ITEM).post(item::post))
}

pub fn join_path(root: String, path: &str) -> String {
    root.trim_end_matches('/').to_owned() +
        "/" + path.trim_start_matches('/')
}

pub fn no_content() -> HttpResponse {
    HttpResponse::new(StatusCode::NO_CONTENT)
}
