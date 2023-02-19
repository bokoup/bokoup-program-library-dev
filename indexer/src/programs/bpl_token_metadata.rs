use crate::{AccountMessageData, TransactionMessageData};
use anchor_lang::{AccountDeserialize, Discriminator};
use bpl_api_data::{
    queries::bpl_token_metadata::{
        burn_delegated_promo_token, campaign, create_campaign, create_promo, delegate_promo_token,
        mint_promo_token, promo, sign_memo,
    },
    Client,
};
pub use bpl_token_metadata::{instruction, state, ID};

#[tracing::instrument(skip_all)]
async fn process_promo<'a>(
    pg_client: &Client,
    key: &[u8],
    buf: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match state::Promo::try_deserialize(buf) {
        Ok(ref account) => promo::upsert(pg_client, key, account, slot, write_version).await,
        Err(error) => {
            tracing::error!(id = bs58::encode(key).into_string(), ?error)
        }
    }
}

#[tracing::instrument(skip_all)]
async fn process_campaign<'a>(
    pg_client: &Client,
    key: &[u8],
    buf: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match state::Campaign::try_deserialize(buf) {
        Ok(ref account) => campaign::upsert(pg_client, key, account, slot, write_version).await,
        Err(error) => {
            tracing::error!(id = bs58::encode(key).into_string(), ?error)
        }
    }
}

pub async fn process<'a>(pg_client: deadpool_postgres::Object, message: AccountMessageData<'a>) {
    let key = message.account.pubkey.as_ref();
    let mut buf = message.account.data.as_ref();
    let slot = message.slot;
    let write_version = message.account.write_version;

    let discriminator = &buf[..8];

    if discriminator == state::Promo::discriminator() {
        process_promo(&pg_client, key, &mut buf, slot, write_version).await
    } else if discriminator == state::Campaign::discriminator() {
        process_campaign(&pg_client, key, &mut buf, slot, write_version).await
    } else {
        ()
    }
}
#[non_exhaustive]
#[derive(Debug)]
pub struct Discriminatorio;

impl Discriminatorio {
    pub const CREATE_CAMPAIGN: [u8; 8] = [249, 176, 197, 218, 167, 92, 64, 22];
    pub const CREATE_PROMO: [u8; 8] = [135, 231, 68, 194, 63, 31, 192, 82];
    pub const MINT_PROMO_TOKEN: [u8; 8] = [75, 139, 89, 205, 32, 105, 163, 161];
    pub const DELEGATE_PROMO_TOKEN: [u8; 8] = [85, 206, 226, 194, 207, 166, 164, 22];
    pub const BURN_DELEGATED_PROMO_TOKEN: [u8; 8] = [119, 36, 30, 56, 83, 96, 21, 132];
    pub const SIGN_MEMO: [u8; 8] = [163, 48, 14, 17, 151, 234, 75, 51];
}

#[tracing::instrument(skip_all)]
pub async fn process_transaction<'a>(
    pg_client: deadpool_postgres::Object,
    message: TransactionMessageData,
) {
    let discriminator = message.data[..8].try_into().unwrap_or([0; 8]);

    match discriminator {
        Discriminatorio::CREATE_CAMPAIGN => {
            create_campaign::upsert(
                &pg_client,
                &message.signature,
                &message.accounts,
                &message.data,
                message.slot,
            )
            .await
        }

        Discriminatorio::CREATE_PROMO => {
            create_promo::upsert(
                &pg_client,
                &message.signature,
                &message.accounts,
                &message.data,
                message.slot,
            )
            .await
        }
        Discriminatorio::MINT_PROMO_TOKEN => {
            mint_promo_token::upsert(
                &pg_client,
                &message.signature,
                &message.accounts,
                &message.data,
                message.slot,
            )
            .await
        }
        Discriminatorio::DELEGATE_PROMO_TOKEN => {
            delegate_promo_token::upsert(
                &pg_client,
                &message.signature,
                &message.accounts,
                &message.data,
                message.slot,
            )
            .await
        }
        Discriminatorio::BURN_DELEGATED_PROMO_TOKEN => {
            burn_delegated_promo_token::upsert(
                &pg_client,
                &message.signature,
                &message.accounts,
                &message.data,
                message.slot,
            )
            .await
        }
        Discriminatorio::SIGN_MEMO => {
            sign_memo::upsert(
                &pg_client,
                &message.signature,
                &message.accounts,
                &message.data,
                message.slot,
            )
            .await
        }
        _ => {
            tracing::info!(
                discriminator = format!("{:?}", discriminator),
                accounts_count = message.accounts.len(),
                message = "not found"
            );
        }
    };
}
