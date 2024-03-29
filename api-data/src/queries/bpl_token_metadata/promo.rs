use bpl_token_metadata::state::Promo;
use tokio_postgres::Client;
use tracing::{error, info};

const UPSERT_QUERY: &str = include_str!("promo_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(client: &Client, key: &[u8], account: &Promo, slot: u64, write_version: u64) {
    let id = bs58::encode(key).into_string();
    let campaign = account.campaign.to_string();
    let mint = account.mint.to_string();
    let metadata = account.metadata.to_string();
    let mint_count = account.mint_count as i32;
    let burn_count = account.burn_count as i32;
    let max_mint = account.max_mint.map(|v| v as i32);
    let max_burn = account.max_burn.map(|v| v as i32);
    let active = account.active;
    let slot = slot as i64;
    let write_version = write_version as i64;

    let result = client
        .query_one(
            UPSERT_QUERY,
            &[
                &id,
                &campaign,
                &mint,
                &metadata,
                &mint_count,
                &burn_count,
                &max_mint,
                &max_burn,
                &active,
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
