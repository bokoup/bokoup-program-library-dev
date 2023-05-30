use anchor_lang::prelude::Pubkey;
use axum::{
    extract::{Extension, Path},
    Json,
};
use serde::{Deserialize, Serialize};
use solana_sdk::{signer::Signer, transaction::Transaction};
use std::{str::FromStr, sync::Arc};

use crate::{
    error::AppError, handlers::SignMemoParams, utils::solana::create_sign_memo_instruction, State,
};

use super::PayResponse;

pub async fn handler(
    Path(SignMemoParams { message, memo }): Path<SignMemoParams>,
    Extension(state): Extension<Arc<State>>,
    Json(data): Json<Data>,
) -> Result<Json<PayResponse>, AppError> {
    let signer = Pubkey::from_str(&data.account)?;
    let payer = state.platform_signer.pubkey();

    tracing::debug!(
        signer = signer.to_string(),
        payer = payer.to_string(),
        memo = memo
    );
    let instruction = create_sign_memo_instruction(payer, memo, signer)?;

    let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer));
    let latest_blockhash = state.solana.get_latest_blockhash().await?;
    tracing::debug!(latest_blockhash = latest_blockhash.to_string());

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
