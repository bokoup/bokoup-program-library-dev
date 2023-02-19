use super::PayResponse;
use crate::{error::AppError, handlers::MintParams, utils::solana::create_mint_promo_instruction};
use anchor_lang::prelude::Pubkey;
use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};
use solana_sdk::transaction::Transaction;
use std::str::FromStr;

pub async fn handler(
    Json(data): Json<Data>,
    Path(MintParams {
        mint,
        location,
        device,
        campaign,
        token_owner,
        message,
        memo,
    }): Path<MintParams>,
) -> Result<Json<PayResponse>, AppError> {
    tracing::debug!(
        mint = mint,
        location = location,
        device = device,
        campaign = campaign,
        message = message,
        memo = memo
    );

    let device_owner = Pubkey::from_str(&data.account)?;
    let mint = Pubkey::from_str(&mint)?;
    let location = Pubkey::from_str(&location)?;
    let device = Pubkey::from_str(&device)?;
    let campaign = Pubkey::from_str(&campaign)?;
    let token_owner = Pubkey::from_str(&token_owner)?;

    let instruction = create_mint_promo_instruction(
        device_owner,
        location,
        device,
        campaign,
        token_owner,
        mint,
        memo,
    )?;

    let tx = Transaction::new_with_payer(&[instruction], Some(&device_owner));
    // let latest_blockhash = state.solana.get_latest_blockhash().await?;
    // tx.try_partial_sign(&[&state.platform_signer], latest_blockhash)?;
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
