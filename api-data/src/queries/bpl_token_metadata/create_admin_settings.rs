use solana_sdk::{pubkey::Pubkey, signature::Signature};
use tokio_postgres::Client;
use tracing::{error, info};

const UPSERT_QUERY: &str = include_str!("create_admin_settings_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(
    client: &Client,
    signature: &Signature,
    accounts: &Vec<Pubkey>,
    _data: &[u8],
    slot: u64,
) {
    let accounts: Vec<String> = accounts.iter().map(ToString::to_string).collect();

    let signature = signature.to_string();
    let payer = &accounts[0];
    let admin_settings = &accounts[1];
    let slot = slot as i64;

    let result = client
        .query_one(UPSERT_QUERY, &[&signature, payer, admin_settings, &slot])
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
