use anchor_lang::prelude::Pubkey;
use axum::{
    extract::{Extension, Path},
    Json,
};
use serde::{Deserialize, Serialize};
use solana_sdk::transaction::Transaction;
use std::{str::FromStr, sync::Arc};

use crate::{error::AppError, utils::solana::create_burn_delegated_promo_instruction, State};

use super::{BurnDelegatedParams, PayResponse};

pub async fn handler(
    Json(data): Json<Data>,
    Path(BurnDelegatedParams {
        mint,
        token_account,
        location,
        device,
        campaign,
        message,
        memo,
    }): Path<BurnDelegatedParams>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<PayResponse>, AppError> {
    let device_owner = Pubkey::from_str(&data.account)?;
    let mint = Pubkey::from_str(&mint)?;
    let token_account = Pubkey::from_str(&token_account)?;
    let location = Pubkey::from_str(&location)?;
    let device = Pubkey::from_str(&device)?;
    let campaign = Pubkey::from_str(&campaign)?;
    let platform = state.platform;

    let instruction = create_burn_delegated_promo_instruction(
        device_owner,
        location,
        device,
        campaign,
        token_account,
        mint,
        platform,
        memo,
    )?;

    let tx = Transaction::new_with_payer(&[instruction], Some(&device_owner));
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
// https://sandbox.dev.clover.com/v3/apps/MAC8DQKWCCB1R/merchants/XKDCJNW9JXGM1/notifications
