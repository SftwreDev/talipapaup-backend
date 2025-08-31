use sea_orm::{FromQueryResult, ModelTrait};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, Set, Statement, TryGetableMany};
use sea_orm::QueryFilter;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use sea_orm::EntityTrait;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde_json::json;
use uuid::Uuid;
use crate::models::carts::{CartsResponse, NewCart};
use crate::models::carts;
use crate::models::prelude::{Carts, Products};
use crate::models::products;
use crate::models::products::ProductsResponse;
use crate::models::responses::{ErrorResponse, SuccessResponse};
use crate::services::{create_new_cart_item, find_existing_cart_item, update_cart_quantity, validate_product_exists};
use crate::utils::local_datetime;

#[post("/carts/")]
pub async fn add_to_cart(
    db: web::Data<sea_orm::DatabaseConnection>,
    new_cart: web::Json<NewCart>,
) -> impl Responder {
    let now: DateTimeWithTimeZone = local_datetime();

    // Validate product exists
    if let Err(response) = validate_product_exists(new_cart.product_id.clone(), db.get_ref()).await {
        return response;
    }

    // Validate quantity
    if new_cart.total_qty <= 0 {
        return HttpResponse::BadRequest().json(ErrorResponse {
            detail: "Quantity must be greater than 0.".to_string(),
        });
    }

    // Check if a product already exists in the user's cart
    match find_existing_cart_item(String::from(new_cart.user_id), new_cart.product_id, db.get_ref()).await {
        Ok(Some(existing_cart)) => {
            // Update existing cart item
            match update_cart_quantity(existing_cart, new_cart.total_qty, now, db.get_ref()).await {
                Ok(updated_cart) => {
                    HttpResponse::Ok().json(SuccessResponse {
                        success: true,
                        message: format!(
                            "Product quantity updated in cart. Added {} items.",
                            new_cart.total_qty
                        ),
                        data: vec![updated_cart],
                    })
                }
                Err(e) => {
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        detail: format!("Unable to update cart quantity: {}", e),
                    })
                }
            }
        }
        Ok(None) => {
            // Create a new cart item
            match create_new_cart_item(
                String::from(new_cart.user_id),
                new_cart.product_id,
                new_cart.total_qty,
                now,
                db.get_ref(),
            ).await {
                Ok(created_cart) => {
                    HttpResponse::Created().json(SuccessResponse {
                        success: true,
                        message: "The product was successfully added to the cart.".to_string(),
                        data: vec![created_cart],
                    })
                }
                Err(e) => {
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        detail: format!("Unable to add product to cart: {}", e),
                    })
                }
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                detail: format!("Database error while checking existing cart: {}", e),
            })
        }
    }
}


#[get("/carts/{user_id}")]
pub async fn get_cart_by_user_id(
    db: web::Data<sea_orm::DatabaseConnection>,
    req: HttpRequest,
) -> impl Responder {
    // 🛠 Extract product_id from a request path
    let user_id_str = match req.match_info().get("user_id") {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "detail": "Invalid or missing user_id."
            }));
        }
    };

    match Carts::find()
        .filter(carts::Column::UserId.eq(user_id_str.to_string()))
        .one(db.get_ref())
        .await
    {
        Ok(Some(carts)) => {


            // Raw SQL query joining carts and products
            let sql = r#"
                SELECT
                    (array_agg(c.id ORDER BY c.created_at))[1] AS id,
                    c.product_id,
                    SUM(c.total_qty)::INTEGER AS total_qty,
                    MIN(c.created_at) AS created_at,
                    MAX(c.updated_at) AS updated_at,
                    p.product_name,
                    p.description,
                    p.price as product_price,
                    (SUM(c.total_qty) * p.price)::NUMERIC AS sub_total_price,
                    p.img_url
                FROM carts c
                INNER JOIN products p ON c.product_id = p.id
                WHERE c.user_id = $1
                GROUP BY c.product_id, p.product_name, p.description, p.price, p.img_url
                ORDER BY c.product_id;
            "#;

            match CartsResponse::find_by_statement(Statement::from_sql_and_values(
                db.get_database_backend(),
                sql,
                vec![user_id_str.into()], // Use parsed user_id
            ))
                .all(db.get_ref())
                .await
            {
                Ok(carts_responses) => {
                    if carts_responses.is_empty() {
                        return HttpResponse::NotFound().json(ErrorResponse {
                            detail: "No carts found for this user.".to_string(),
                        });
                    }

                    HttpResponse::Ok().json(SuccessResponse {
                        success: true,
                        message: "Carts fetched successfully.".to_string(),
                        data: carts_responses,
                    })
                }
                Err(e) => {
                    eprintln!("❌ Error fetching carts: {}", e);
                    HttpResponse::InternalServerError().json(json!({
                "detail": "Failed to fetch carts."
            }))
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            detail: "Carts not found.".to_string(),
        }),
        Err(e) => {
            eprintln!("❌ Error fetching carts: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "detail": e.to_string()
            }))
        }
    }
}

#[put("/carts/qty/{user_id}/{product_id}/{qty}/")]
pub async fn update_cart_qty(
    db: web::Data<sea_orm::DatabaseConnection>,
    req: HttpRequest,
) -> impl Responder {
    // 🛠 Extract user_id, product_id and qty from a request path
    let user_id = match req.match_info().get("user_id") {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "detail": "Invalid or missing user_id."
            }));
        }
    };

    let product_id = match req.match_info().get("product_id") {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "detail": "Invalid or missing product_id."
            }));
        }
    };

    let qty_str = match req.match_info().get("qty") {
        Some(qty) => qty,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "detail": "Invalid or missing qty."
            }));
        }
    };

    // Parse qty to integer
    let qty: i32 = match qty_str.parse() {
        Ok(q) => q,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                detail: "Invalid quantity format. Must be a number.".to_string(),
            });
        }
    };

    // Validate qty is positive
    if qty <= 0 {
        return HttpResponse::BadRequest().json(ErrorResponse {
            detail: "Quantity must be greater than 0.".to_string(),
        });
    }

    // Parse product_id (assuming it's a string or UUID)
    let parsed_product_id = match product_id.parse() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                detail: "Invalid product_id format.".to_string(),
            });
        }
    };

    // Validate product exists
    if let Err(response) = validate_product_exists(*&parsed_product_id, db.get_ref()).await {
        return response;
    }


    // Find and update cart item
    match find_existing_cart_item(user_id.parse().unwrap(), parsed_product_id, db.get_ref()).await {
        Ok(Some(cart_item)) => {
            // Update the cart item
            let now = local_datetime();
            let mut cart_active_model: carts::ActiveModel = cart_item.into();
            cart_active_model.total_qty = Set(qty);
            cart_active_model.updated_at = Set(now);

            match cart_active_model.update(db.get_ref()).await {
                Ok(updated_cart) => {
                    HttpResponse::Ok().json(SuccessResponse {
                        success: true,
                        message: "Cart quantity updated successfully.".to_string(),
                        data: updated_cart,
                    })
                }
                Err(e) => {
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        detail: format!("Database error while updating cart: {}", e),
                    })
                }
            }
        }
        Ok(None) => {
            HttpResponse::NotFound().json(ErrorResponse {
                detail: format!(
                    "No cart item found for user '{}' with product_id '{}'.",
                    user_id,
                    product_id
                ),
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                detail: format!("Database error while finding cart item: {}", e),
            })
        }
    }
}

#[delete("/carts/{user_id}/{product_id}")]
pub async fn delete_cart_item(
    db: web::Data<sea_orm::DatabaseConnection>,
    req: HttpRequest,
) -> impl Responder {
    // 🛠 Extract user_id and product_id from a request path
    let user_id = match req.match_info().get("user_id") {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "detail": "Invalid or missing user_id."
            }));
        }
    };

    let product_id = match req.match_info().get("product_id") {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "detail": "Invalid or missing product_id."
            }));
        }
    };

    // Parse product_id (assuming it's a string or UUID)
    let parsed_product_id = match product_id.parse() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                detail: "Invalid product_id format.".to_string(),
            });
        }
    };

    // Optional: Validate product exists (you might skip this for delete operations)
    if let Err(response) = validate_product_exists(*&parsed_product_id, db.get_ref()).await {
        return response;
    }

    // Find the cart item to delete
    match carts::Entity::find()
        .filter(carts::Column::UserId.eq(user_id))
        .filter(carts::Column::ProductId.eq(*&parsed_product_id))
        .one(db.get_ref())
        .await
    {
        Ok(Some(cart_item)) => {
            // Store the item details before deletion (for response)

            // Delete the cart item
            match cart_item.delete(db.get_ref()).await {
                Ok(_delete_result) => {
                    HttpResponse::Ok().json(SuccessResponse {
                        success: true,
                        message: format!(
                            "Cart item successfully deleted for user '{}' and product '{}'.",
                            user_id,
                            product_id
                        ),
                        data: "None",
                    })
                }
                Err(e) => {
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        detail: format!("Database error while deleting cart item: {}", e),
                    })
                }
            }
        }
        Ok(None) => {
            HttpResponse::NotFound().json(ErrorResponse {
                detail: format!(
                    "No cart item found for user '{}' with product_id '{}'.",
                    user_id,
                    product_id
                ),
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                detail: format!("Database error while finding cart item: {}", e),
            })
        }
    }
}


#[delete("/carts/{user_id}")]
pub async fn delete_all_cart_item_per_user_id(
    db: web::Data<sea_orm::DatabaseConnection>,
    req: HttpRequest,
) -> impl Responder {
    let user_id = match req.match_info().get("user_id") {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                detail: "Invalid or missing user_id.".to_string(),
            });
        }
    };

    // Delete using bulk delete operation
    match carts::Entity::find()
        .filter(carts::Column::UserId.eq(user_id))
        .one(db.get_ref())
        .await
    {
        Ok(Some(cart_item)) => {
            // Store the item details before deletion (for response)

            // Delete the cart item
            match cart_item.delete(db.get_ref()).await {
                Ok(_delete_result) => {
                    HttpResponse::Ok().json(SuccessResponse {
                        success: true,
                        message: format!(
                            "Cart item successfully deleted for user '{}'.",
                            user_id,
                        ),
                        data: "None",
                    })
                }
                Err(e) => {
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        detail: format!("Database error while deleting cart item: {}", e),
                    })
                }
            }
        }
        Ok(None) => {
            HttpResponse::NotFound().json(ErrorResponse {
                detail: format!(
                    "No cart item found for user '{}'.",
                    user_id
                ),
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                detail: format!("Database error while finding cart item: {}", e),
            })
        }
    }
}