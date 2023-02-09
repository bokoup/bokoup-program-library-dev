use bpl_token_metadata::state::Campaign;
use tokio_postgres::{types::Json, Client};
use tracing::{error, info};

const UPSERT_QUERY: &str = include_str!("campaign_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(
    client: &Client,
    key: &[u8],
    account: &Campaign,
    slot: u64,
    write_version: u64,
) {
    let id = bs58::encode(key).into_string();
    let merchant = account.merchant.to_string();
    let locations = account.locations.iter().map(ToString::to_string).collect();
    let slot = slot as i64;
    let write_version = write_version as i64;

    let result = client
        .query_one(
            UPSERT_QUERY,
            &[
                &id,
                &merchant,
                &account.name,
                &account.name,
                &Json::<Vec<String>>(locations),
                &account.active,
                &slot,
                &write_version,
            ],
        )
        .await;
    match result {
        Ok(row) => {
            let insert = row.get::<usize, Option<bool>>(0).unwrap();
            info!(id = id.as_str(), insert);
        }
        Err(error) => {
            error!(id = id.as_str(), ?error);
        }
    }
}
