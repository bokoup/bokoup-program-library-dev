use crate::{
    error::AppError,
    utils::{
        bundlr::{upload_image, upload_metadata_json},
        multipart::{get_args, get_metadata},
        solana::create_location_instruction,
    },
    State,
};
use anchor_lang::prelude::Pubkey;
use axum::{
    extract::{Multipart, Path},
    Extension, Json,
};
use solana_sdk::{signature::Signer, transaction::Transaction};
use std::{str::FromStr, sync::Arc};

use super::{LocationParams, PayResponse};

pub async fn handler(
    Path(LocationParams { owner, memo }): Path<LocationParams>,
    Extension(state): Extension<Arc<State>>,
    multipart: Multipart,
) -> Result<Json<PayResponse>, AppError> {
    let payer = state.platform_signer.pubkey();

    // Parse data - two parts - json data and image.
    let (mut metadata_data, image_data) = get_metadata(multipart).await?;

    let metadata_data_obj =
        metadata_data
            .as_object_mut()
            .ok_or(AppError::MultipartMetadataError(
                "metadata data part should be an object".to_string(),
            ))?;

    // Parse merchant args.
    let (name, active) = get_args(metadata_data_obj)?;
    metadata_data_obj.remove("active");

    // If image exists, upload to arweave and add uri to metadata.
    let state = if let Some(image_data) = image_data {
        let (image_url, _, state) = upload_image(image_data, state).await?;
        metadata_data_obj.insert("image".to_string(), image_url.into());
        state
    } else {
        state
    };

    // Upload metadata json to Arweave.
    let (uri, state) = upload_metadata_json(metadata_data_obj, state).await?;

    let owner = Pubkey::from_str(&owner)?;

    // Create location instruction.
    let ix = create_location_instruction(payer, owner, name, uri, active, memo)?;

    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer));
    let latest_blockhash = &state.solana.get_latest_blockhash().await?;
    tx.try_partial_sign(&[&state.platform_signer], latest_blockhash.clone())?;

    let serialized = bincode::serialize(&tx)?;
    let transaction = base64::encode(serialized);

    Ok(Json(PayResponse {
        transaction,
        message: "Create location".to_string(),
    }))
}
