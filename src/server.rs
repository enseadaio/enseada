use actix_web::{App, HttpServer};
use actix_web::middleware::Logger;

use crate::routes::routes;

pub async fn run() -> std::io::Result<()> {
    HttpServer::new(move ||
        App::new()
            .wrap(Logger::default())
            .configure(routes)
    )
        .bind("0.0.0.0:9623")?
        .run()
        .await
}