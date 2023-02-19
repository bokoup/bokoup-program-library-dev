use crate::{
    error::AppError,
    utils::{
        bundlr::{upload_image, upload_metadata_json},
        multipart::{get_args, get_metadata},
        solana::create_campaign_instruction,
    },
    State,
};
use anchor_lang::prelude::Pubkey;
use axum::{
    extract::{Multipart, Path},
    Extension, Json,
};
use solana_sdk::{pubkey::ParsePubkeyError, transaction::Transaction};
use std::{str::FromStr, sync::Arc};

use super::{CampaignParams, PayResponse};

pub async fn handler(
    multipart: Multipart,
    Path(CampaignParams {
        payer,
        lamports,
        memo,
        locations,
    }): Path<CampaignParams>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<PayResponse>, AppError> {
    tracing::debug!(
        payer = payer,
        lamports = lamports,
        memo = memo,
        locations = locations
    );

    let payer = Pubkey::from_str(&payer)?;

    let locations: Vec<Pubkey> = locations
        .split("/")
        .filter(|s| s != &"")
        .map(Pubkey::from_str)
        .collect::<Vec<Result<Pubkey, ParsePubkeyError>>>()
        .into_iter()
        .collect::<Result<Vec<Pubkey>, ParsePubkeyError>>()?;

    // Parse metadata - leaving option of image in the future.
    let (mut metadata_data, image_data) = get_metadata(multipart).await?;

    let metadata_data_obj =
        metadata_data
            .as_object_mut()
            .ok_or(AppError::MultipartMetadataError(
                "metadata data part should be an object".to_string(),
            ))?;

    // // Parse args.
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
    let uri = upload_metadata_json(metadata_data_obj, state).await?.0;

    // Create location instruction.
    let ix = create_campaign_instruction(payer, name, uri, locations, lamports, active, memo)?;

    let tx = Transaction::new_with_payer(&[ix], Some(&payer));

    let serialized = bincode::serialize(&tx)?;
    let transaction = base64::encode(serialized);

    Ok(Json(PayResponse {
        transaction,
        message: "Create campaign".to_string(),
    }))
}
