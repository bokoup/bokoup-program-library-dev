use bpl_token_metadata::state::CampaignLocation;
use tokio_postgres::Client;
use tracing::{error, info};

const UPSERT_QUERY: &str = include_str!("campaign_location_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(
    client: &Client,
    key: &[u8],
    account: &CampaignLocation,
    slot: u64,
    write_version: u64,
) {
    let id = bs58::encode(key).into_string();
    let campaign = account.campaign.to_string();
    let location = account.location.to_string();
    let slot = slot as i64;
    let write_version = write_version as i64;

    let result = client
        .query_one(
            UPSERT_QUERY,
            &[&id, &campaign, &location, &slot, &write_version],
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
