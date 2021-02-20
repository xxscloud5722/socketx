#![feature(try_trait)]

#[macro_use]
extern crate thiserror;

mod router;
mod server;
mod data;

use crate::server::web;



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("./config/log.yaml", Default::default()).expect("log error");
    web::server().await
}

#[test]
fn test() {
    println!("V:: {}", server::config::get_env("${JAVA_HOME}").unwrap());
}