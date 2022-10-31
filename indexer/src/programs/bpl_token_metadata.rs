use crate::{AccountMessageData, TransactionMessageData};
use anchor_lang::AccountDeserialize;
use bpl_api_data::{
    queries::bpl_token_metadata::{
        burn_delegated_promo_token, create_promo, create_promo_group, delegate_promo_token,
        mint_promo_token, promo, promo_group,
    },
    Client,
};
pub use bpl_token_metadata::{
    state::{Promo, PromoGroup},
    ID,
};

#[tracing::instrument(skip_all)]
async fn process_promo<'a>(
    pg_client: &Client,
    key: &[u8],
    buf: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match Promo::try_deserialize(buf) {
        Ok(ref account) => promo::upsert(pg_client, key, account, slot, write_version).await,
        Err(error) => {
            tracing::error!(id = bs58::encode(key).into_string(), ?error)
        }
    }
}

#[tracing::instrument(skip_all)]
async fn process_promo_group<'a>(
    pg_client: &Client,
    key: &[u8],
    buf: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match PromoGroup::try_deserialize(buf) {
        Ok(ref account) => promo_group::upsert(pg_client, key, account, slot, write_version).await,
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

    match buf.len() {
        Promo::LEN => process_promo(&pg_client, key, &mut buf, slot, write_version).await,
        PromoGroup::LEN => {
            process_promo_group(&pg_client, key, &mut buf, slot, write_version).await
        }
        _ => (),
    }
}
#[non_exhaustive]
#[derive(Debug)]
pub struct Discriminator;

impl Discriminator {
    pub const CREATE_PROMO_GROUP: [u8; 8] = [249, 176, 197, 218, 167, 92, 64, 22];
    pub const CREATE_PROMO: [u8; 8] = [135, 231, 68, 194, 63, 31, 192, 82];
    pub const MINT_PROMO_TOKEN: [u8; 8] = [75, 139, 89, 205, 32, 105, 163, 161];
    pub const DELEGATE_PROMO_TOKEN: [u8; 8] = [85, 206, 226, 194, 207, 166, 164, 22];
    pub const BURN_DELEGATED_PROMO_TOKEN: [u8; 8] = [119, 36, 30, 56, 83, 96, 21, 132];
}

#[tracing::instrument(skip_all)]
pub async fn process_transaction<'a>(
    pg_client: deadpool_postgres::Object,
    message: TransactionMessageData,
) {
    let discriminator = message.data[..8].try_into().unwrap_or([0; 8]);

    match discriminator {
        Discriminator::CREATE_PROMO_GROUP => {
            create_promo_group::upsert(
                &pg_client,
                &message.signature,
                &message.accounts,
                &message.data,
                message.slot,
            )
            .await
        }

        Discriminator::CREATE_PROMO => {
            create_promo::upsert(
                &pg_client,
                &message.signature,
                &message.accounts,
                &message.data,
                message.slot,
            )
            .await
        }
        Discriminator::MINT_PROMO_TOKEN => {
            mint_promo_token::upsert(
                &pg_client,
                &message.signature,
                &message.accounts,
                &message.data,
                message.slot,
            )
            .await
        }
        Discriminator::DELEGATE_PROMO_TOKEN => {
            delegate_promo_token::upsert(
                &pg_client,
                &message.signature,
                &message.accounts,
                &message.data,
                message.slot,
            )
            .await
        }
        Discriminator::BURN_DELEGATED_PROMO_TOKEN => {
            burn_delegated_promo_token::upsert(
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
