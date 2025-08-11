use serde::{Deserialize, Serialize};

// Success response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: T,
}

// Error response schema
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub detail: String,
}
