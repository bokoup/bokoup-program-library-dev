use anchor_lang::prelude::Pubkey;
use axum::{
    extract::{Extension, Path},
    Json,
};
use serde::{Deserialize, Serialize};
use solana_sdk::{signer::Signer, transaction::Transaction};
use std::{str::FromStr, sync::Arc};

use crate::{
    error::AppError, handlers::Params, utils::solana::create_mint_promo_instruction, State,
    PROMO_GROUP_QUERY,
};

pub async fn handler(
    Json(data): Json<Data>,
    Path(Params {
        mint_string,
        message,
        memo,
    }): Path<Params>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<ResponseData>, AppError> {
    tracing::debug!(mint_string = mint_string, message = message, memo = memo);

    let token_owner = Pubkey::from_str(&data.account)?;
    let payer = state.promo_owner.pubkey();
    let mint = Pubkey::from_str(&mint_string)?;

    // TODO: make request to data api to confirm platform address is included in
    // members of group.
    let group_seed = Pubkey::new_unique();
    let instruction = create_mint_promo_instruction(payer, group_seed, token_owner, mint, memo)?;

    let query = serde_json::json!({ "mint": mint_string, "query": PROMO_GROUP_QUERY, "variables": {"mint": mint_string}});

    let result: serde_json::Value = state
        .solana
        .client
        .post(&state.data_url)
        .json(&query)
        .send()
        .await?
        .json()
        .await?;

    tracing::debug!("{:?}", result.to_string());

    let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer));
    let latest_blockhash = state.solana.get_latest_blockhash().await?;
    tx.try_partial_sign(&[&state.promo_owner], latest_blockhash)?;
    let serialized = bincode::serialize(&tx)?;
    let transaction = base64::encode(serialized);

    Ok(Json(ResponseData {
        transaction,
        message,
    }))
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Data {
    pub account: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResponseData {
    pub transaction: String,
    pub message: String,
}
