use anchor_lang::prelude::Pubkey;
use axum::{
    extract::{Extension, Path},
    Json,
};
use serde::{Deserialize, Serialize};
use solana_sdk::{signer::Signer, transaction::Transaction};
use std::{str::FromStr, sync::Arc};

use crate::{
    error::AppError, handlers::DelegateParams, utils::solana::create_delegate_promo_instruction,
    State,
};

use super::PayResponse;

// Payer is distinct from
pub async fn handler(
    Json(data): Json<Data>,
    Path(DelegateParams {
        mint,
        device_owner,
        location,
        device,
        campaign,
        message,
        memo,
    }): Path<DelegateParams>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<PayResponse>, AppError> {
    let token_owner = Pubkey::from_str(&data.account)?;
    let payer = state.platform_signer.pubkey();
    let mint = Pubkey::from_str(&mint)?;
    let device_owner = Pubkey::from_str(&device_owner)?;
    let location = Pubkey::from_str(&location)?;
    let device = Pubkey::from_str(&device)?;
    let campaign = Pubkey::from_str(&campaign)?;

    let instruction = create_delegate_promo_instruction(
        payer,
        device_owner,
        location,
        device,
        campaign,
        token_owner,
        mint,
        memo,
    )?;

    let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer));
    let recent_blockhash = state.solana.get_latest_blockhash().await?;
    tx.try_partial_sign(&[&state.platform_signer], recent_blockhash)?;

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

// https://sandbox.dev.clover.com/v3/apps/MAC8DQKWCCB1R/merchants/XKDCJNW9JXGM1/notifications
