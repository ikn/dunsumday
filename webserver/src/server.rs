use std::net::ToSocketAddrs;
use std::net::Ipv4Addr;
use actix_web::{App, HttpServer, middleware, web};
use dunsumday::config::{self, Config};
use dunsumday::db::Db;
use crate::{api, configrefs, ui};

pub struct State {
    pub db: Box<dyn Db>,
}

impl State {
    pub fn new(cfg: Box<dyn Config>) -> Result<State, String> {
        let db = dunsumday::db::open(cfg.as_ref())?;
        Ok::<State, String>(State {
            db: Box::new(db),
        })
    }
}

pub trait ConfigFactory {
    fn get(&self) -> Result<Box<dyn Config>, String>;
}

pub fn addr<C>(cfg: &C) -> Result<impl ToSocketAddrs, String>
where
    C: Config + ?Sized,
{
    let all_interfaces = config::get_ref(cfg, &configrefs::SERVER_ALL_INTERFACES)?;
    let addr = if all_interfaces { Ipv4Addr::UNSPECIFIED }
               else { Ipv4Addr::LOCALHOST };
    let port = config::get_ref(cfg, &configrefs::SERVER_PORT)?;
    Ok((addr, port))
}

#[actix_web::main]
pub async fn run<F>(cfg_factory: &'static F) -> Result<(), String>
where
    F: ConfigFactory + Sync
{
    HttpServer::new(|| {
        let app = App::new()
            .data_factory(|| async {
                State::new(cfg_factory.get()?)
            })
            .wrap(middleware::Logger::default())
            .default_service(web::to(api::notfound::get));

        // no way to handle errors properly here
        let cfg = cfg_factory.get().unwrap();
        let root_path = config::get_ref(cfg.as_ref(), &configrefs::SERVER_ROOT_PATH)
            .unwrap()
            .trim_end_matches('/').to_string();
        let api_service = api::service(cfg.as_ref()).unwrap();
        let ui_service = ui::service(cfg.as_ref()).unwrap();
        app.service(web::scope(&root_path)
            .service(api_service).service(ui_service))
    })
        .bind_auto_h2c(addr(cfg_factory.get()?.as_ref()).unwrap())
        .map_err(|e| format!("error binding port: {e}"))?
        .run()
        .await
        .map_err(|e| format!("error initialising or interrupted: {e}"))
}
