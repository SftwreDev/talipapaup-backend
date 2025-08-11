use crate::models::prelude::Products;
use crate::models::products;
use crate::models::products::{NewProduct, ProductsResponse};
use crate::models::responses::{ErrorResponse, SuccessResponse};
use crate::utils::local_datetime;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{ActiveModelTrait, ColumnTrait, QueryOrder};
use sea_orm::{EntityTrait, Set};
use sea_orm::{Order, QueryFilter};
use serde_json::json;
use uuid::Uuid;

/// Create a new product
///
/// - Validates that no product with the same name exists (case-insensitive).
/// - Normalizes the product name to lowercase before saving.
/// - Inserts the product with current timestamps.
/// - Returns `201 Created` with product details if successful.
#[post("/products/")]
pub async fn create_product(
    db: web::Data<sea_orm::DatabaseConnection>,
    new_product: web::Json<NewProduct>,
) -> impl Responder {
    let now: DateTimeWithTimeZone = local_datetime();
    let normalized_name = new_product.product_name.trim().to_lowercase();

    // 🔍 Check if a product with the same normalized name already exists
    match products::Entity::find()
        .filter(products::Column::ProductName.eq(normalized_name.clone()))
        .one(db.get_ref())
        .await
    {
        Ok(Some(_)) => {
            return HttpResponse::Conflict().json(ErrorResponse {
                detail: "A product with this name already exists.".to_string(),
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                detail: format!("Database error while checking for duplicate: {}", e),
            });
        }
        Ok(None) => {}
    }

    // 🏗️ Construct the new product ActiveModel
    let new_product_model = products::ActiveModel {
        id: Set(Uuid::new_v4()),
        product_name: Set(normalized_name),
        description: Set(new_product.description.clone()),
        price: Set(new_product.price),
        category: Set(new_product.category.clone()),
        img_url: Set(new_product.img_url.clone()),
        is_available: Set(new_product.is_available),
        created_at: Set(now),
        updated_at: Set(now),
    };

    // 💾 Insert the new product into the database
    match new_product_model.insert(db.get_ref()).await {
        Ok(created_product) => HttpResponse::Created().json(SuccessResponse {
            success: true,
            message: "Product created successfully.".to_string(),
            data: vec![created_product], // Could map to a ProductResponse DTO if needed
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            detail: format!("Failed to create product: {}", e),
        }),
    }
}

/// Fetch all products
///
/// - Returns products ordered by creation date (descending).
/// - Returns `404 Not Found` if there are no products.
/// - On success, returns a list of products.
#[get("/products")]
pub async fn fetch_products(db: web::Data<sea_orm::DatabaseConnection>) -> impl Responder {
    match Products::find()
        .order_by(products::Column::CreatedAt, Order::Desc)
        .all(db.get_ref())
        .await
    {
        Ok(products) => {
            if products.is_empty() {
                return HttpResponse::NotFound().json(ErrorResponse {
                    detail: "No products found.".to_string(),
                });
            }

            let products_responses: Vec<ProductsResponse> = products
                .into_iter()
                .map(ProductsResponse::from_model)
                .collect();

            HttpResponse::Ok().json(SuccessResponse {
                success: true,
                message: "Products fetched successfully.".to_string(),
                data: products_responses,
            })
        }
        Err(e) => {
            eprintln!("❌ Error fetching products: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                detail: format!("Failed to fetch products: {}", e),
            })
        }
    }
}

/// Fetch a single product by ID
///
/// - Validates the UUID format.
/// - Returns `404 Not Found` if the product doesn't exist.
/// - On success, returns the product details.
#[get("/products/{product_id}")]
pub async fn fetch_product_by_id(
    db: web::Data<sea_orm::DatabaseConnection>,
    req: HttpRequest,
) -> impl Responder {
    // 🛠 Extract product_id from a request path
    let product_id_str = match req.match_info().get("product_id") {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "detail": "Invalid or missing product_id."
            }));
        }
    };

    // 🔍 Validate and parse the UUID
    let product_uuid = match Uuid::parse_str(product_id_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({
                "detail": "Invalid product_id format."
            }));
        }
    };

    // 📦 Fetch the product from the database
    match Products::find()
        .filter(products::Column::Id.eq(product_uuid))
        .one(db.get_ref())
        .await
    {
        Ok(Some(product)) => {
            let products_responses = vec![ProductsResponse::from_model(product)];

            HttpResponse::Ok().json(SuccessResponse {
                success: true,
                message: "Product fetched successfully.".to_string(),
                data: products_responses,
            })
        }
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            detail: "Product not found.".to_string(),
        }),
        Err(e) => {
            eprintln!("❌ Error fetching product: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "detail": e.to_string()
            }))
        }
    }
}
