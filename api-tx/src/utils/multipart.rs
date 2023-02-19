use crate::error::AppError;
use axum::extract::Multipart;
use serde_json::{Map, Value};

/// Returns metadata and image data if image is a multipart field.
pub async fn get_metadata(
    mut multipart: Multipart,
) -> Result<(Value, Option<(Vec<u8>, String)>), AppError> {
    let metadata_data = if let Some(field) = multipart.next_field().await.unwrap() {
        if field.name().expect("metadata field should exist") == "metadata" {
            let json_string = field.text().await.map_err(|_| {
                AppError::MultipartMetadataError("metadata value not valid".to_string())
            })?;
            Ok(serde_json::from_str::<Value>(&json_string)?)
        } else {
            return Err(AppError::MultipartMetadataError(
                "invalid field name".to_string(),
            ));
        }
    } else {
        Err(AppError::MultipartMetadataError(
            "request had no parts".to_string(),
        ))
    }?;

    let image_data = if let Some(field) = multipart.next_field().await.unwrap() {
        if field.name().expect("image field should exist") == "image" {
            let content_type = field.content_type().map(ToString::to_string).ok_or(
                AppError::MultipartImageError("failed to read image content type".to_string()),
            )?;
            let image_bytes = field.bytes().await.map_err(|_| {
                AppError::MultipartImageError("failed to read image bytes".to_string())
            })?;
            Some((image_bytes.to_vec(), content_type))
        } else {
            return Err(AppError::MultipartImageError(
                "invalid field name".to_string(),
            ));
        }
    } else {
        None
    };
    Ok((metadata_data, image_data))
}

pub fn get_args(metadata_data_obj: &mut Map<String, Value>) -> Result<(String, bool), AppError> {
    let name = metadata_data_obj["name"]
        .as_str()
        .ok_or(AppError::MultipartMetadataError(
            "name field should exist".to_string(),
        ))?
        .to_string();

    let active = metadata_data_obj["active"]
        .as_bool()
        .ok_or(AppError::MultipartMetadataError(
            "active field should exist".to_string(),
        ))?;

    Ok((name, active))
}

pub fn get_promo_args(
    metadata_data_obj: &mut Map<String, Value>,
) -> Result<(String, String, Option<u32>, Option<u32>, bool), AppError> {
    let name = metadata_data_obj["name"]
        .as_str()
        .ok_or(AppError::CreatePromoRequestError(
            "name field should exist".to_string(),
        ))?
        .to_string();

    let symbol = metadata_data_obj["symbol"]
        .as_str()
        .ok_or(AppError::CreatePromoRequestError(
            "symbol field should exist".to_string(),
        ))?
        .to_string();

    let active = metadata_data_obj["active"]
        .as_bool()
        .ok_or(AppError::MultipartMetadataError(
            "active field should exist".to_string(),
        ))?;

    // Return max_mint and max_burn if attributes exists in json data.
    let (max_mint, max_burn) = if let Some(value) = metadata_data_obj.get("attributes") {
        if let Some(attributes) = value.as_array() {
            let max_mint: Option<u32> = attributes
                .iter()
                .filter_map(|a| {
                    let attribute = a.as_object()?;
                    if let Some(trait_type) = attribute.get("trait_type") {
                        if trait_type == "maxMint" {
                            attribute.get("value").map(|v| v.as_u64()).unwrap_or(None)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<u64>>()
                .first()
                .map(|v| v.clone() as u32);

            let max_burn: Option<u32> = attributes
                .iter()
                .filter_map(|a| {
                    let attribute = a.as_object()?;
                    if let Some(trait_type) = attribute.get("trait_type") {
                        if trait_type == "maxBurn" {
                            attribute.get("value").map(|v| v.as_u64()).unwrap_or(None)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<u64>>()
                .first()
                .map(|v| v.clone() as u32);

            (max_mint, max_burn)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };
    Ok((name, symbol, max_mint, max_burn, active))
}
