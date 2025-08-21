use actix_web::{web, HttpResponse};
use sea_orm::{DatabaseConnection, QueryFilter};
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use uuid::Uuid;
use crate::models::products;
use crate::models::responses::ErrorResponse;

// Function to find a product by ID
pub async fn find_product_by_id(
    product_id: Uuid,
    db: &DatabaseConnection,
) -> Result<Option<products::Model>, sea_orm::DbErr> {
    products::Entity::find()
        .filter(products::Column::Id.eq(product_id))
        .one(db)
        .await
}

// Function to handle product validation and return the appropriate HTTP response
pub async fn validate_product_exists(
    product_id: Uuid,
    db: &DatabaseConnection,
) -> Result<(), HttpResponse> {
    match find_product_by_id(product_id, db).await {
        Ok(None) => {
            Err(HttpResponse::Conflict().json(ErrorResponse {
                detail: "No product found with this ID.".to_string(),
            }))
        }
        Ok(Some(_)) => Ok(()),
        Err(e) => {
            Err(HttpResponse::InternalServerError().json(ErrorResponse {
                detail: format!("Database error while checking product: {}", e),
            }))
        }
    }
}
