use crate::{AccountMessageData, TransactionMessageData};
use anchor_lang::{AccountDeserialize, Discriminator};
use bpl_api_data::{
    queries::bpl_token_metadata::{
        admin_settings, burn_delegated_promo_token, campaign, campaign_location,
        create_admin_settings, create_campaign, create_campaign_location, create_device,
        create_location, create_merchant, create_promo, delegate_promo_token, device, location,
        merchant, mint_promo_token, promo, sign_memo,
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
async fn process_admin_settings<'a>(
    pg_client: &Client,
    key: &[u8],
    buf: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match state::AdminSettings::try_deserialize(buf) {
        Ok(ref account) => {
            admin_settings::upsert(pg_client, key, account, slot, write_version).await
        }
        Err(error) => {
            tracing::error!(id = bs58::encode(key).into_string(), ?error)
        }
    }
}

#[tracing::instrument(skip_all)]
async fn process_merchant<'a>(
    pg_client: &Client,
    key: &[u8],
    buf: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match state::Merchant::try_deserialize(buf) {
        Ok(ref account) => merchant::upsert(pg_client, key, account, slot, write_version).await,
        Err(error) => {
            tracing::error!(id = bs58::encode(key).into_string(), ?error)
        }
    }
}

#[tracing::instrument(skip_all)]
async fn process_location<'a>(
    pg_client: &Client,
    key: &[u8],
    buf: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match state::Location::try_deserialize(buf) {
        Ok(ref account) => location::upsert(pg_client, key, account, slot, write_version).await,
        Err(error) => {
            tracing::error!(id = bs58::encode(key).into_string(), ?error)
        }
    }
}

#[tracing::instrument(skip_all)]
async fn process_device<'a>(
    pg_client: &Client,
    key: &[u8],
    buf: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match state::Device::try_deserialize(buf) {
        Ok(ref account) => device::upsert(pg_client, key, account, slot, write_version).await,
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

#[tracing::instrument(skip_all)]
async fn process_campaign_location<'a>(
    pg_client: &Client,
    key: &[u8],
    buf: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match state::CampaignLocation::try_deserialize(buf) {
        Ok(ref account) => {
            campaign_location::upsert(pg_client, key, account, slot, write_version).await
        }
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

    if discriminator == state::AdminSettings::discriminator() {
        process_admin_settings(&pg_client, key, &mut buf, slot, write_version).await
    } else if discriminator == state::Merchant::discriminator() {
        process_merchant(&pg_client, key, &mut buf, slot, write_version).await
    } else if discriminator == state::Location::discriminator() {
        process_location(&pg_client, key, &mut buf, slot, write_version).await
    } else if discriminator == state::Device::discriminator() {
        process_device(&pg_client, key, &mut buf, slot, write_version).await
    } else if discriminator == state::Campaign::discriminator() {
        process_campaign(&pg_client, key, &mut buf, slot, write_version).await
    } else if discriminator == state::CampaignLocation::discriminator() {
        process_campaign_location(&pg_client, key, &mut buf, slot, write_version).await
    } else if discriminator == state::Promo::discriminator() {
        process_promo(&pg_client, key, &mut buf, slot, write_version).await
    } else {
        ()
    }
}
#[non_exhaustive]
#[derive(Debug)]
pub struct Discriminatorio;

impl Discriminatorio {
    pub const CREATE_ADMIN_SETTINGS: [u8; 8] = [122, 229, 242, 6, 66, 19, 86, 127];
    pub const CREATE_MERCHANT: [u8; 8] = [249, 172, 245, 100, 32, 117, 97, 156];
    pub const CREATE_LOCATION: [u8; 8] = [46, 89, 192, 49, 76, 189, 44, 8];
    pub const CREATE_DEVICE: [u8; 8] = [56, 101, 5, 177, 25, 113, 80, 174];
    pub const CREATE_CAMPAIGN: [u8; 8] = [111, 131, 187, 98, 160, 193, 114, 244];
    pub const CREATE_CAMPAIGN_LOCATION: [u8; 8] = [82, 9, 70, 52, 189, 11, 188, 239];
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
        Discriminatorio::CREATE_ADMIN_SETTINGS => {
            create_admin_settings::upsert(
                &pg_client,
                &message.signature,
                &message.accounts,
                &message.data,
                message.slot,
            )
            .await
        }
        Discriminatorio::CREATE_MERCHANT => {
            create_merchant::upsert(
                &pg_client,
                &message.signature,
                &message.accounts,
                &message.data,
                message.slot,
            )
            .await
        }

        Discriminatorio::CREATE_LOCATION => {
            create_location::upsert(
                &pg_client,
                &message.signature,
                &message.accounts,
                &message.data,
                message.slot,
            )
            .await
        }

        Discriminatorio::CREATE_DEVICE => {
            create_device::upsert(
                &pg_client,
                &message.signature,
                &message.accounts,
                &message.data,
                message.slot,
            )
            .await
        }

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

        Discriminatorio::CREATE_CAMPAIGN_LOCATION => {
            create_campaign_location::upsert(
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
                &message.balances,
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
                &message.balances,
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
