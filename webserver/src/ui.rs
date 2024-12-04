use actix_web::web;
use actix_web::dev::HttpServiceFactory;
use dunsumday::config::Config;
use crate::configrefs;

pub fn service<C>(cfg: &C) -> impl HttpServiceFactory
where
    C: Config + ?Sized,
{
    let files = actix_files::Files::new(
            cfg.get_ref(&configrefs::SERVER_UI_PATH),
            cfg.get_ref(&configrefs::UI_PATH)
    )
        .index_file("index.html")
        .redirect_to_slash_directory();
    web::scope("")
        .service(files)
}
