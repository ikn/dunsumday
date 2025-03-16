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

pub fn addr<'a, C>(cfg: &'a C) -> Result<impl ToSocketAddrs, String>
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
pub async fn run<C>(api_enabled: bool, ui_enabled: bool, cfg: &'static C)
-> Result<(), String>
where
    &'static C: Send,
    C: Config + Clone + Send,
{
    let addr = addr(cfg).unwrap();

    HttpServer::new(move || {
        let app = App::new()
            .data_factory(|| async {
                State::new(Box::new(cfg.clone()))
            })
            .wrap(middleware::Logger::default())
            .default_service(web::to(api::notfound::get));

        // no way to handle errors properly here
        let root_path = config::get_ref(cfg, &configrefs::SERVER_ROOT_PATH)
            .unwrap()
            .trim_end_matches('/').to_string();
        let service = web::scope(&root_path);
        let service = if api_enabled {
            service.service(api::service(cfg).unwrap())
        } else { service };
        let service = if ui_enabled {
            service.service(ui::service(cfg).unwrap())
        } else { service };
        app.service(service)
    })
        .bind_auto_h2c(addr)
        .map_err(|e| format!("error binding port: {e}"))?
        .run()
        .await
        .map_err(|e| format!("error initialising or interrupted: {e}"))
}
