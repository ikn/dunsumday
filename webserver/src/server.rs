use std::{borrow::Borrow, net::ToSocketAddrs};
use std::net::Ipv4Addr;
use dunsumday::config::Config;
use dunsumday::db::Db;
use crate::configrefs;

pub struct State {
    pub cfg: Box<dyn Config>,
    pub db: Box<dyn Db>,
}

impl State {
    pub fn new(cfg: Box<dyn Config>) -> Result<State, String> {
        let db = dunsumday::db::open(cfg.borrow() as &dyn Config)?;
        Ok::<State, String>(State {
            cfg,
            db: Box::new(db),
        })
    }
}

pub fn addr<C>(cfg: &C) -> impl ToSocketAddrs
where
    C: Config + ?Sized,
{
    let all_interfaces = cfg.get_ref(&configrefs::SERVER_ALL_INTERFACES);
    let addr = if all_interfaces == "true" { Ipv4Addr::UNSPECIFIED }
               else { Ipv4Addr::LOCALHOST };
    (addr, cfg.get_ref(&configrefs::SERVER_PORT).parse().unwrap())
}
