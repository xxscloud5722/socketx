use log::info;
use actix_web::{HttpServer, web, App};
use crate::server::{socket, config};
use crate::router;


pub async fn server() -> std::io::Result<()> {
    info!("[Actix]: Start Web Server ...");
    let port = config::get_i64("port").unwrap();
    HttpServer::new(|| {
        App::new()
            .route("/ws/*", web::get().to(socket::ws))
            .configure(|config| {
                router::route::configure(config)
            })
    })
        .bind(format!(":::{}", port))
        .expect("web sever error")
        .run()
        .await
}

