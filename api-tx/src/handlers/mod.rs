use serde::{Deserialize, Serialize};

pub mod get_app_id;
pub mod get_burn_delegated_promo_tx;
pub mod get_create_campaign_tx;
pub mod get_create_device_tx;
pub mod get_create_location_tx;
pub mod get_create_merchant_tx;
pub mod get_create_promo_tx;
pub mod get_delegate_promo_tx;
pub mod get_mint_promo_tx;
pub mod get_sign_memo_tx;

#[derive(Deserialize, Debug)]
pub struct MintParams {
    pub mint: String,
    pub device: String,
    pub device_owner: String,
    pub location: String,
    pub campaign: String,
    pub message: String,
    pub memo: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct DelegateParams {
    pub mint: String,
    pub device_owner: String,
    pub device: String,
    pub location: String,
    pub campaign: String,
    pub message: String,
    pub memo: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct BurnDelegatedParams {
    pub mint: String,
    pub token_account: String,
    pub device: String,
    pub location: String,
    pub campaign: String,
    pub message: String,
    pub memo: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct PromoParams {
    pub owner: String,
    pub campaign: String,
    pub memo: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SignMemoParams {
    pub memo: String,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct BasicParams {
    pub payer: String,
    pub memo: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct MerchantParams {
    pub owner: String,
    pub memo: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct LocationParams {
    pub owner: String,
    pub memo: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct DeviceParams {
    pub merchant_owner: String,
    pub location: String,
    pub owner: String,
    pub memo: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CampaignParams {
    pub owner: String,
    pub lamports: u64,
    pub memo: Option<String>,
    pub locations: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PayResponse {
    pub transaction: String,
    pub message: String,
}
