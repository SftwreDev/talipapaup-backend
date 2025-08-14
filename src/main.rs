mod services;

use crate::handlers::categories::delete_category;
use crate::handlers::{add_category, create_product, delete_product, fetch_categories, fetch_product_by_id, fetch_products, update_product};
use crate::services::establish_connection;
use actix_cors::Cors;
use actix_web::{get, middleware::Logger as ActixLogger, web, HttpResponse, Responder};
use colourful_logger::Logger;
use shuttle_actix_web::ShuttleActixWeb;

mod handlers;
mod models;
mod utils;

#[get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut web::ServiceConfig) + Send + Clone + 'static> {
    // Remove dotenv - Shuttle handles environment variables
    let logger = Logger::default();

    logger.info_single("🚀 Starting Actix server on Shuttle", "SERVER");

    // 💾 Connect to the database
    let db = establish_connection().await;

    let config = move |cfg: &mut web::ServiceConfig| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        cfg.service(
            web::scope("/api/v1")
                .app_data(web::Data::new(db.clone()))
                .wrap(ActixLogger::default())
                .wrap(cors)
                .service(healthz)
                // Categories endpoints
                .service(add_category)
                .service(fetch_categories)
                .service(delete_category)
                // Products endpoints
                .service(create_product)
                .service(fetch_products)
                .service(fetch_product_by_id)
                .service(update_product)
                .service(delete_product)
        );
    };

    Ok(config.into())
}
