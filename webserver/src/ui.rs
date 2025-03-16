use actix_web::web;
use actix_web::dev::HttpServiceFactory;
use dunsumday::config::{self, Config};
use crate::configrefs;

pub fn service<C>(cfg: &C) -> Result<impl HttpServiceFactory, String>
where
    C: Config + ?Sized,
{
    let ui_path = config::get_ref(cfg, &configrefs::UI_PATH)?;
    let files = actix_files::Files::new("/", &ui_path)
        .index_file("index.html")
        .redirect_to_slash_directory();

    Ok(web::scope(&config::get_ref(cfg, &configrefs::SERVER_UI_PATH)?)
        .service(files))
}
