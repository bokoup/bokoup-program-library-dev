use super::PayResponse;
use crate::{error::AppError, handlers::MintParams, utils::solana::mint_promo_instruction, State};
use anchor_lang::prelude::Pubkey;
use axum::{extract::Path, Extension, Json};
use serde::{Deserialize, Serialize};
use solana_sdk::{signer::Signer, transaction::Transaction};
use std::{str::FromStr, sync::Arc};

pub async fn handler(
    Json(data): Json<Data>,
    Path(MintParams {
        mint,
        device,
        device_owner,
        location,
        campaign,
        message,
        memo,
    }): Path<MintParams>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<PayResponse>, AppError> {
    tracing::debug!(
        mint = mint,
        device = device,
        device_owner = device_owner,
        location = location,
        campaign = campaign,
        message = message,
        memo = memo
    );

    let payer = state.platform_signer.pubkey();
    let device_owner = Pubkey::from_str(&device_owner)?;
    let mint = Pubkey::from_str(&mint)?;
    let device = Pubkey::from_str(&device)?;
    let campaign = Pubkey::from_str(&campaign)?;
    let location = Pubkey::from_str(&location)?;
    let token_owner = Pubkey::from_str(&data.account)?;

    let instruction = mint_promo_instruction(
        payer,
        device_owner,
        device,
        location,
        campaign,
        token_owner,
        mint,
        memo,
    )?;

    let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer));
    let latest_blockhash = state.solana.get_latest_blockhash().await?;

    // platform_signer signs as payer and also as device_owner if device_owner is platform_signer.
    tx.try_partial_sign(&[&state.platform_signer], latest_blockhash)?;

    let serialized = bincode::serialize(&tx)?;
    let transaction = base64::encode(serialized);

    Ok(Json(PayResponse {
        transaction,
        message,
    }))
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Data {
    pub account: String,
}
