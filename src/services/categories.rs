use crate::models::categories;
use crate::models::prelude::Categories;
use actix_web::web;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use uuid::Uuid;

pub async fn fetch_category_by_id(
    db: web::Data<sea_orm::DatabaseConnection>,
    category_id: String,
) -> String {
    let category_uuid = match Uuid::parse_str(&category_id) {
        Ok(id) => id,
        Err(_) => return "".to_string(), // invalid UUID
    };

    match Categories::find()
        .filter(categories::Column::Id.eq(category_uuid))
        .one(db.get_ref())
        .await
    {
        Ok(Some(category)) => category.name, // adjust if your column name is different
        _ => "".to_string(),
    }
}
