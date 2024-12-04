use std::fmt::Debug;
use actix_web::error::ErrorInternalServerError;
use actix_web::{web, Responder};
use serde::{Deserialize, Serialize};
use dunsumday::db::SortDirection;
use crate::{constant, api, server};

#[derive(Debug, Deserialize, Serialize)]
pub struct Item { name: String }

#[derive(Debug, Deserialize, Serialize)]
pub struct NewItem { name: String }

pub async fn list(data: web::Data<server::State>)
-> actix_web::Result<impl Responder> {
    let items = data.db
        .find_items(
            Some(true), None, SortDirection::Asc, constant::ITEMS_PAGE_SIZE)
        .map_err(|e| ErrorInternalServerError(e))?
        .into_iter()
        .map(|item| Item { name: item.item.name })
        .collect::<Vec<_>>();
    Ok(web::Json(items))
}

pub async fn post(_item: web::Json<NewItem>)
-> actix_web::Result<impl Responder> {
    Ok(api::no_content())
}
