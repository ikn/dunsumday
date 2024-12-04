use std::borrow::Borrow;
use actix_web::{App, HttpServer, middleware, web};
use dunsumday::config::{self, Config};

mod configrefs;
mod constant;
mod api;
mod ui;
mod server;

fn cfg_factory() -> Result<Box<dyn Config>, String> {
    // /usr/local/etc/dunsumday/config.yaml
    const CONFIG_PATH: &str = "dev-config.yaml";
    Ok(Box::new(config::file::new(CONFIG_PATH)?))
}

#[actix_web::main]
async fn main() -> Result<(), String> {
    env_logger::init();

    let global_cfg = cfg_factory()?;
    HttpServer::new(|| {
        let app = App::new()
            .data_factory(|| async {
                server::State::new(cfg_factory()?)
            })
            .wrap(middleware::Logger::default())
            .default_service(web::to(api::notfound::get));

        // no way to handle errors properly here
        let cfg = cfg_factory().unwrap();
        let root_path = cfg.get_ref(&configrefs::SERVER_ROOT_PATH)
            .trim_end_matches('/');
        let api_service = api::service(cfg.borrow() as &dyn Config);
        let ui_service = ui::service(cfg.borrow() as &dyn Config);
        app.service(web::scope(root_path)
            .service(api_service).service(ui_service))
    })
        .bind_auto_h2c(server::addr(global_cfg.borrow() as &dyn Config))
        .map_err(|e| format!("error binding port: {e}"))?
        .run()
        .await
        .map_err(|e| format!("error initialising or interrupted: {e}"))
}
