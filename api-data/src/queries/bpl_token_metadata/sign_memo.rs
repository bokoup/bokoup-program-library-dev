use borsh::de::BorshDeserialize;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use tokio_postgres::{types::Json, Client};
use tracing::{error, info};

const UPSERT_QUERY: &str = include_str!("sign_memo_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(
    client: &Client,
    signature: &Signature,
    accounts: &Vec<Pubkey>,
    data: &[u8],
    slot: u64,
) {
    let accounts: Vec<String> = accounts.iter().map(ToString::to_string).collect();
    let memo =
        if let Ok(args) = bpl_token_metadata::instruction::SignMemo::try_from_slice(&data[8..]) {
            if let Ok(result) = serde_json::from_str::<serde_json::Value>(&args.memo) {
                result
            } else {
                serde_json::json!({ "memo": args.memo.to_string()})
            }
        } else {
            serde_json::Value::String("".to_string())
        };

    let signature = signature.to_string();
    let payer = &accounts[0];
    let signer = &accounts[1];
    let slot = slot as i64;

    let result = client
        .query_one(
            UPSERT_QUERY,
            &[
                &signature,
                payer,
                signer,
                &Json::<serde_json::Value>(memo),
                &slot,
            ],
        )
        .await;
    match result {
        Ok(row) => {
            let insert = row.get::<usize, Option<bool>>(0).unwrap();
            info!(signature = signature.as_str(), insert);
        }
        Err(error) => {
            error!(signature = signature.as_str(), ?error,);
        }
    }
}
