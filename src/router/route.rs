use actix_web::{web};
use crate::router::{api};
use actix_web::web::ServiceConfig;

pub fn configure(config: &mut ServiceConfig) {
    config.route("/send", web::post().to(api::send));
    config.route("/get_list",web::post().to(api::get_list));
}