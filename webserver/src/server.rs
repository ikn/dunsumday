use std::net::ToSocketAddrs;
use std::net::Ipv4Addr;
use dunsumday::config::{self, Config};
use dunsumday::db::Db;
use crate::configrefs;

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

pub fn addr<C>(cfg: &C) -> Result<impl ToSocketAddrs, String>
where
    C: Config + ?Sized,
{
    let all_interfaces = config::get_ref(cfg, &configrefs::SERVER_ALL_INTERFACES)?;
    let addr = if all_interfaces == "true" { Ipv4Addr::UNSPECIFIED }
               else { Ipv4Addr::LOCALHOST };
    let port = config::get_ref(cfg, &configrefs::SERVER_PORT)?
        .parse::<u16>()
        .map_err(|e| format!("error parsing port number: {e}"))?;
    Ok((addr, port))
}
