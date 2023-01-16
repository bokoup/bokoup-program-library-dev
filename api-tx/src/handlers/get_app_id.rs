use crate::error::AppError;
use axum::Json;
use serde::{Deserialize, Serialize};

pub const LABEL: &str = "bokoup";
pub const ICON: &str = "https://arweave.net/wrKmRzr2KhH92c1iyFeUqkB-AHjYlE7Md7U5rK4qA8M";

pub async fn handler() -> Result<Json<ResponseData>, AppError> {
    Ok(Json(ResponseData {
        label: LABEL.to_string(),
        icon: ICON.to_string(),
    }))
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResponseData {
    pub label: String,
    pub icon: String,
}
