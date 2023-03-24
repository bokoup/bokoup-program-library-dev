use borsh::de::BorshDeserialize;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use tokio_postgres::{types::Json, Client};
use tracing::{error, info};

const UPSERT_QUERY: &str = include_str!("burn_delegated_promo_token_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(
    client: &Client,
    signature: &Signature,
    accounts: &Vec<Pubkey>,
    balances: &Vec<u64>,
    data: &[u8],
    slot: u64,
) {
    let accounts: Vec<String> = accounts.iter().map(ToString::to_string).collect();
    let memo = if let Ok(args) =
        bpl_token_metadata::instruction::MintPromoToken::try_from_slice(&data[8..])
    {
        args.memo.map(|m| {
            if let Ok(result) = serde_json::from_str::<serde_json::Value>(&m) {
                result
            } else {
                serde_json::json!({ "memo": m })
            }
        })
    } else {
        None
    };

    let signature = signature.to_string();
    let payer = &accounts[0];
    let location = &accounts[1];
    let device = &accounts[2];
    let campaign = &accounts[3];
    let mint = &accounts[4];
    let authority = &accounts[5];
    let promo = &accounts[6];
    let platform = &accounts[7];
    let admin_settings = &accounts[8];
    let token_account = &accounts[9];
    let slot = slot as i64;

    let payer_balance = balances[0] as i64;
    let campaign_balance = balances[3] as i64;
    let platform_balance = balances[7] as i64;

    let result = client
        .query_one(
            UPSERT_QUERY,
            &[
                &signature,
                payer,
                &payer_balance,
                location,
                device,
                campaign,
                &campaign_balance,
                mint,
                authority,
                promo,
                platform,
                &platform_balance,
                admin_settings,
                token_account,
                &Json::<Option<serde_json::Value>>(memo),
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
            error!(signature = signature.as_str(), ?error);
        }
    }
}
