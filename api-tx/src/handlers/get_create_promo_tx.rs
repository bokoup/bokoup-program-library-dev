use crate::{
    error::AppError,
    utils::{
        bundlr::{upload_image, upload_metadata_json},
        multipart::{get_metadata, get_promo_args},
        solana::create_promo_instruction,
    },
    State,
};
use anchor_lang::prelude::Pubkey;
use axum::{
    extract::{Multipart, Path},
    Extension, Json,
};
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use std::{str::FromStr, sync::Arc};

use super::{PayResponse, PromoParams};

#[axum_macros::debug_handler]
pub async fn handler(
    Path(PromoParams {
        owner,
        campaign,
        memo,
    }): Path<PromoParams>,
    Extension(state): Extension<Arc<State>>,
    multipart: Multipart,
) -> Result<Json<PayResponse>, AppError> {
    tracing::debug!(owner = owner, campaign = campaign, memo = memo,);

    // Parse data - json data plus optional image. If image data exists it gets
    // uploaded to arweave and an image property added to the json metadata.
    let (mut metadata_data, image_data) = get_metadata(multipart).await?;

    let metadata_data_obj =
        metadata_data
            .as_object_mut()
            .ok_or(AppError::MultipartMetadataError(
                "metadata data part should be an object".to_string(),
            ))?;

    // Parse args.
    let (name, symbol, max_mint, max_burn, active) = get_promo_args(metadata_data_obj)?;
    metadata_data_obj.remove("active");
    metadata_data_obj.remove("max_mint");
    metadata_data_obj.remove("max_burn");

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

    let mint_keypair = Keypair::new();
    let payer = state.platform_signer.pubkey();
    let owner = Pubkey::from_str(&owner)?;
    let campaign = Pubkey::from_str(&campaign)?;

    // Create promo instruction.
    let ix = create_promo_instruction(
        payer,
        owner,
        campaign,
        mint_keypair.pubkey(),
        state.platform,
        name,
        symbol,
        uri,
        max_mint,
        max_burn,
        active,
        true,
        memo,
    )?;

    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer));
    let latest_blockhash = state.solana.get_latest_blockhash().await?;
    tx.partial_sign(&[&state.platform_signer, &mint_keypair], latest_blockhash);

    let serialized = bincode::serialize(&tx)?;
    let transaction = base64::encode(serialized);

    Ok(Json(PayResponse {
        transaction,
        message: "Create promo".to_string(),
    }))
}
