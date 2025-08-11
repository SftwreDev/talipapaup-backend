use crate::models::categories;
use crate::models::categories::{CategoryResponse, NewCategory};
use crate::models::prelude::Categories;
use crate::models::responses::{ErrorResponse, SuccessResponse};
use crate::utils::local_datetime;
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{ActiveModelTrait, DeleteResult, EntityTrait, Set};
use sea_orm::{ColumnTrait, Order, QueryOrder};
use sea_orm::{DatabaseConnection, QueryFilter};
use serde_json::json;
use uuid::Uuid;

/// Adds a new category to the database.
///
/// # Endpoint
/// `POST /category/`
///
/// # Request
/// Accepts a JSON body conforming to `NewCategory`.
///
/// # Response
/// - 201 Created: If the category is successfully created.
/// - 409 Conflict: If a category with the same name already exists.
/// - 500 Internal Server Error: On database-related failures.
#[post("/category/")]
pub async fn add_category(
    db: web::Data<sea_orm::DatabaseConnection>,
    new_category: web::Json<NewCategory>,
) -> impl Responder {
    let now: DateTimeWithTimeZone = local_datetime();
    let normalized_name = new_category.name.trim().to_lowercase();

    // Check if a category with the same name already exists in the database
    match Categories::find()
        .filter(categories::Column::Name.eq(normalized_name.clone()))
        .one(db.get_ref())
        .await
    {
        Ok(Some(_)) => {
            // Category already exists, return 409 Conflict
            return HttpResponse::Conflict().json(ErrorResponse {
                detail: "Category with this name already exists".to_string(),
            });
        }
        Err(e) => {
            // Database query failed, return 500 Internal Server Error
            return HttpResponse::InternalServerError().json(ErrorResponse {
                detail: format!("Database error: {}", e),
            });
        }
        Ok(None) => {} // Category doesn't exist, proceed to creation
    }

    // Construct a new category ActiveModel
    let new_category_model = categories::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(normalized_name),
        created_at: Set(now),
        updated_at: Set(now),
    };

    // Attempt to insert the new category into the database
    match new_category_model.insert(db.get_ref()).await {
        Ok(created_category) => {
            // Successfully created category, return 201 Created
            let category_response = CategoryResponse::from_model(created_category);
            HttpResponse::Created().json(SuccessResponse {
                success: true,
                message: "Category created successfully".to_string(),
                data: vec![category_response],
            })
        }
        Err(e) => {
            // Insert operation failed, return 500 Internal Server Error
            HttpResponse::InternalServerError().json(ErrorResponse {
                detail: format!("Failed to create category: {}", e),
            })
        }
    }
}
/// Fetches all categories from the database.
///
/// # Endpoint
/// `GET /category`
///
/// # Response
/// - 200 OK: Returns a list of categories.
/// - 404 Not Found: If no categories are found.
/// - 500 Internal Server Error: If a database error occurs.
#[get("/category")]
pub async fn fetch_categories(db: web::Data<sea_orm::DatabaseConnection>) -> impl Responder {
    // Query the database for all categories, ordered by creation date descending
    match Categories::find()
        .order_by(categories::Column::CreatedAt, Order::Desc)
        .all(db.get_ref())
        .await
    {
        Ok(categories) => {
            if categories.is_empty() {
                // Return 404 if no categories found
                return HttpResponse::NotFound().json(ErrorResponse {
                    detail: "No categories found".to_string(),
                });
            }

            // Map database models to response format
            let category_responses: Vec<CategoryResponse> = categories
                .into_iter()
                .map(CategoryResponse::from_model)
                .collect();

            HttpResponse::Ok().json(SuccessResponse {
                success: true,
                message: "Categories fetched successfully".to_string(),
                data: category_responses,
            })
        }
        Err(e) => {
            // Log and return 500 error on failure
            eprintln!("❌ Error fetching categories: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                detail: format!("Failed to fetch categories: {}", e),
            })
        }
    }
}

#[delete("/category/{category_id}")]
pub async fn delete_category(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
) -> impl Responder {
    let category_id = match req.match_info().get("category_id") {
        Some(id) => match Uuid::parse_str(id) {
            Ok(parsed_id) => parsed_id,
            Err(_) => {
                return HttpResponse::BadRequest().json(json!({
                    "detail": "Invalid UUID format for category_id"
                }));
            }
        },
        None => {
            return HttpResponse::BadRequest().json(json!({
                "detail": "Missing category_id"
            }));
        }
    };

    let res: DeleteResult = match Categories::delete_by_id(category_id)
        .exec(db.get_ref())
        .await
    {
        Ok(result) => result,
        Err(e) => {
            eprintln!("❌ Error deleting category record: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "detail": format!("Failed to delete category record: {}", e)
            }));
        }
    };

    if res.rows_affected == 0 {
        return HttpResponse::NotFound().json(json!({
            "detail": "Category record not found"
        }));
    }

    // Return success response
    HttpResponse::Ok().json(json!({
        "detail": "Category record deleted successfully"
    }))
}
