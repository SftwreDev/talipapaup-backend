use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use sea_orm::prelude::DateTimeWithTimeZone;
use uuid::Uuid;
use crate::models::carts;

pub async fn find_existing_cart_item(
    user_id: String,
    product_id: Uuid,
    db: &DatabaseConnection,
) -> Result<Option<carts::Model>, sea_orm::DbErr> {
    carts::Entity::find()
        .filter(carts::Column::UserId.eq(user_id))
        .filter(carts::Column::ProductId.eq(product_id))
        .one(db)
        .await
}

pub async fn update_cart_quantity(
    existing_cart: carts::Model,
    additional_qty: i32,
    now: DateTimeWithTimeZone,
    db: &DatabaseConnection,
) -> Result<carts::Model, sea_orm::DbErr> {
    let mut cart_active_model: carts::ActiveModel = existing_cart.into();
    let current_qty = cart_active_model.total_qty.clone().unwrap();

    cart_active_model.total_qty = Set(current_qty + additional_qty);
    cart_active_model.updated_at = Set(now);

    cart_active_model.update(db).await
}

pub async fn create_new_cart_item(
    user_id: String,
    product_id: Uuid,
    total_qty: i32,
    now: DateTimeWithTimeZone,
    db: &DatabaseConnection,
) -> Result<carts::Model, sea_orm::DbErr> {
    let new_cart_model = carts::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id.to_string()),
        product_id: Set(product_id),
        total_qty: Set(total_qty),
        created_at: Set(now),
        updated_at: Set(now),
    };

    new_cart_model.insert(db).await
}