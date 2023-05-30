use crate::{error::AppError, State};
use bundlr_sdk::{consts::BUNDLR_DEFAULT_URL, currency::CurrencyType, tags::Tag};
use serde_json::{json, Map, Value};
use solana_sdk::signer::Signer;
use std::str::FromStr;
use std::sync::Arc;

pub async fn upload_image(
    image_data: (Vec<u8>, String),
    state: Arc<State>,
) -> Result<(String, String, Arc<State>), AppError> {
    // Upload image to Arweave.
    let mut tx = state.bundlr.create_transaction(
        image_data.0.clone(),
        vec![
            Tag::new("User-Agent".into(), "bokoup".into()),
            Tag::new("Content-Type".into(), &image_data.1),
        ],
    )?;

    let balance = bundlr_sdk::bundlr::get_balance(
        &url::Url::from_str(BUNDLR_DEFAULT_URL).unwrap(),
        CurrencyType::Solana,
        &state.platform_signer.pubkey().to_string(),
        &reqwest::Client::new(),
    )
    .await?;

    tracing::debug!(balance = format!("{}", &balance));

    let mut tx = state.bundlr.create_transaction(
        image_data.0,
        vec![
            Tag::new("User-Agent".into(), "bokoup".into()),
            Tag::new("Content-Type".into(), &image_data.1),
        ],
    )?;

    state.bundlr.sign_transaction(&mut tx).await?;

    // Get id of uploaded image and add to metdata json.
    let response = state.bundlr.send_transaction(tx).await?;
    let image_id = response.as_object().ok_or(AppError::BundlrResponseError(
        "bundlr respsonse should be an object".to_string(),
    ))?;

    let image_url = format!(
        "https://arweave.net/{}",
        image_id["id"]
            .as_str()
            .ok_or(AppError::BundlrResponseError(
                "id field should exist in bundlr response".to_string(),
            ))?
    );

    tracing::debug!(image_url = &image_url);

    Ok((image_url, image_data.1, state))
}

pub async fn upload_metadata_json(
    metadata_data_obj: &mut Map<String, Value>,
    state: Arc<State>,
) -> Result<(String, Arc<State>), AppError> {
    // Upload json metadata to Arweave and get id back for inclusion in creation of on chain Promo.
    let mut tx = state.bundlr.create_transaction(
        serde_json::to_vec(metadata_data_obj)?,
        vec![
            Tag::new("User-Agent".into(), "bokoup".into()),
            Tag::new("Content-Type".into(), "application/json"),
        ],
    )?;

    let balance = bundlr_sdk::bundlr::get_balance(
        &url::Url::from_str(BUNDLR_DEFAULT_URL).unwrap(),
        CurrencyType::Solana,
        &state.platform_signer.pubkey().to_string(),
        &reqwest::Client::new(),
    )
    .await?;

    let price = bundlr_sdk::bundlr::get_price(
        &url::Url::from_str(BUNDLR_DEFAULT_URL).unwrap(),
        CurrencyType::Solana,
        &reqwest::Client::new(),
        tx.as_bytes().unwrap().len() as u64,
    )
    .await?;

    tracing::debug!(
        balance = format!("{}", &balance),
        // price = format!("{}", &price)
    );

    let mut tx = state.bundlr.create_transaction(
        serde_json::to_vec(metadata_data_obj)?,
        vec![
            Tag::new("User-Agent".into(), "bokoup".into()),
            Tag::new("Content-Type".into(), "application/json"),
        ],
    )?;

    state.bundlr.sign_transaction(&mut tx).await?;

    let response = state.bundlr.send_transaction(tx).await?;
    let metadata_id = response.as_object().ok_or(AppError::BundlrResponseError(
        "bundlr respsone should be an object".to_string(),
    ))?;

    let uri = format!(
        "https://arweave.net/{}",
        metadata_id["id"]
            .as_str()
            .ok_or(AppError::BundlrResponseError(
                "id field should exist in bundlr response".to_string(),
            ))?
    );

    tracing::debug!(metadata_json_uri = &uri);

    Ok((uri, state))
}

pub async fn update_promo_metadata_json(
    metadata_data_obj: &mut Map<String, Value>,
    image_url: String,
    content_type: String,
) -> Result<&mut Map<String, Value>, AppError> {
    metadata_data_obj.insert("image".to_string(), image_url.clone().into());

    metadata_data_obj.insert(
        "properties".to_string(),
        json!({
            "files": [{
                "uri": image_url,
                "type": content_type
            }],
            "category": "image"
        }),
    );

    Ok(metadata_data_obj)
}

pub fn update_metadata_json(
    metadata_data_obj: &mut Map<String, Value>,
    image_url: String,
) -> Result<(), AppError> {
    metadata_data_obj.insert("image".to_string(), image_url.clone().into());
    metadata_data_obj.remove("active");
    Ok(())
}
