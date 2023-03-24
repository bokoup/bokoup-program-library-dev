use bpl_token_metadata::state::AdminSettings;
use tokio_postgres::Client;
use tracing::{error, info};

const UPSERT_QUERY: &str = include_str!("admin_settings_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(
    client: &Client,
    key: &[u8],
    account: &AdminSettings,
    slot: u64,
    write_version: u64,
) {
    let id = bs58::encode(key).into_string();
    let platform = account.platform.to_string();
    let create_promo_lamports = account.create_promo_lamports as i64;
    let burn_promo_token_lamports = account.burn_promo_token_lamports as i64;
    let slot = slot as i64;
    let write_version = write_version as i64;

    let result = client
        .query_one(
            UPSERT_QUERY,
            &[
                &id,
                &platform,
                &create_promo_lamports,
                &burn_promo_token_lamports,
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
