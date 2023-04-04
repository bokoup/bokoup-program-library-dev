use crate::error::AppError;
use anchor_lang::{
    prelude::Pubkey,
    InstructionData, ToAccountMetas,
    {
        solana_program::{instruction::Instruction, sysvar},
        system_program,
    },
};
use bpl_token_metadata::{
    accounts::{
        BurnDelegatedPromoToken as burn_delegated_promo_token_accounts,
        CreateCampaign as create_campaign_accounts,
        CreateCampaignLocation as create_campaign_location_accounts,
        CreateDevice as create_device_accounts, CreateLocation as create_location_accounts,
        CreateMerchant as create_merchant_accounts, CreatePromo as create_promo_accounts,
        DelegatePromoToken as delegate_promo_token_accounts,
        MintPromoToken as mint_promo_token_accounts, SignMemo as sign_memo_accounts,
    },
    instruction::{
        BurnDelegatedPromoToken as burn_delegated_promo_token_instruction,
        CreateCampaign as create_campaign_instruction,
        CreateCampaignLocation as create_campaign_location_instruction,
        CreateDevice as create_device_instruction, CreateLocation as create_location_instruction,
        CreateMerchant as create_merchant_instruction, CreatePromo as create_promo_instruction,
        DelegatePromoToken as delegate_promo_token_instruction,
        MintPromoToken as mint_promo_token_instruction, SignMemo as sign_memo_instruction,
    },
    state::{Campaign, DataV2, Device, Location, Merchant, Promo},
    utils::{
        find_admin_address, find_associated_token_address, find_authority_address,
        find_campaign_address, find_campaign_location_address, find_device_address,
        find_location_address, find_merchant_address, find_metadata_address, find_promo_address,
    },
};
use serde::{Deserialize, Serialize};

use serde_json::{json, Value};
use solana_sdk::{commitment_config::CommitmentLevel, hash::Hash};
use std::str::FromStr;

pub fn create_merchant_instruction(
    payer: Pubkey,
    owner: Pubkey,
    name: String,
    uri: String,
    active: bool,
    memo: Option<String>,
) -> Result<Instruction, AppError> {
    let merchant = find_merchant_address(&owner).0;

    let data = Merchant {
        owner,
        name,
        uri,
        active,
    };

    let accounts = create_merchant_accounts {
        payer,
        owner,
        merchant,
        memo_program: spl_memo::ID,
        rent: sysvar::rent::id(),
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let data = create_merchant_instruction { data, memo }.data();

    Ok(Instruction {
        program_id: bpl_token_metadata::id(),
        accounts,
        data,
    })
}

pub fn create_location_instruction(
    payer: Pubkey,
    owner: Pubkey,
    name: String,
    uri: String,
    active: bool,
    memo: Option<String>,
) -> Result<Instruction, AppError> {
    let (merchant, _) = find_merchant_address(&owner);
    let (location, _) = find_location_address(&owner, &name);

    let data = Location {
        merchant,
        name,
        uri,
        active,
    };

    let accounts = create_location_accounts {
        payer,
        owner,
        merchant,
        location,
        memo_program: spl_memo::ID,
        rent: sysvar::rent::id(),
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let data = create_location_instruction { data, memo }.data();

    Ok(Instruction {
        program_id: bpl_token_metadata::id(),
        accounts,
        data,
    })
}

pub fn create_device_instruction(
    payer: Pubkey,
    merchant_owner: Pubkey,
    location: Pubkey,
    owner: Pubkey,
    name: String,
    uri: String,
    active: bool,
    memo: Option<String>,
) -> Result<Instruction, AppError> {
    let (merchant, _) = find_merchant_address(&merchant_owner);
    let (device, _) = find_device_address(&location, &name);

    let data = Device {
        owner,
        location,
        name,
        uri,
        active,
    };

    let accounts = create_device_accounts {
        payer,
        merchant_owner,
        merchant,
        location,
        device,
        memo_program: spl_memo::ID,
        rent: sysvar::rent::id(),
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let data = create_device_instruction { data, memo }.data();

    Ok(Instruction {
        program_id: bpl_token_metadata::id(),
        accounts,
        data,
    })
}

pub fn create_campaign_instruction(
    payer: Pubkey,
    owner: Pubkey,
    name: String,
    uri: String,
    lamports: u64,
    locations: Vec<Pubkey>,
    active: bool,
    memo: Option<String>,
) -> Result<Vec<Instruction>, AppError> {
    let merchant = find_merchant_address(&owner).0;
    let campaign = find_campaign_address(&merchant, &name).0;

    let data = Campaign {
        merchant,
        name,
        uri,
        active,
    };

    let accounts = create_campaign_accounts {
        payer,
        owner,
        merchant,
        campaign,
        memo_program: spl_memo::ID,
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let data = create_campaign_instruction {
        data,
        lamports,
        memo,
    }
    .data();

    let mut instructions: Vec<Instruction> = Vec::new();

    instructions.push(Instruction {
        program_id: bpl_token_metadata::id(),
        accounts,
        data,
    });

    // Create campaign location instructions.
    for location in locations {
        let ix = create_campaign_location_instruction(payer, owner, campaign, location, None)?;
        instructions.push(ix);
    }

    Ok(instructions)
}

pub fn create_campaign_location_instruction(
    payer: Pubkey,
    owner: Pubkey,
    campaign: Pubkey,
    location: Pubkey,
    memo: Option<String>,
) -> Result<Instruction, AppError> {
    let merchant = find_merchant_address(&owner).0;
    let campaign_location = find_campaign_location_address(&campaign, &location).0;

    let accounts = create_campaign_location_accounts {
        payer,
        owner,
        merchant,
        campaign,
        campaign_location,
        location,
        memo_program: spl_memo::ID,
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let data = create_campaign_location_instruction { memo }.data();

    Ok(Instruction {
        program_id: bpl_token_metadata::id(),
        accounts,
        data,
    })
}

pub fn create_promo_instruction(
    payer: Pubkey,
    owner: Pubkey,
    campaign: Pubkey,
    mint: Pubkey,
    platform: Pubkey,
    name: String,
    symbol: String,
    uri: String,
    max_mint: Option<u32>,
    max_burn: Option<u32>,
    active: bool,
    is_mutable: bool,
    memo: Option<String>,
) -> Result<Instruction, AppError> {
    let authority = find_authority_address().0;
    let promo = find_promo_address(&mint).0;
    let metadata = find_metadata_address(&mint).0;
    let admin_settings = find_admin_address().0;
    let merchant = find_merchant_address(&owner).0;

    let accounts = create_promo_accounts {
        payer,
        owner,
        merchant,
        campaign,
        mint,
        metadata,
        authority,
        promo,
        platform,
        admin_settings,
        metadata_program: mpl_token_metadata::ID,
        token_program: anchor_spl::token::ID,
        memo_program: spl_memo::ID,
        rent: sysvar::rent::id(),
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let promo_data = Promo {
        campaign,
        mint,
        metadata,
        mint_count: 0,
        burn_count: 0,
        max_mint,
        max_burn,
        active,
    };

    let metadata_data = DataV2 {
        name,
        symbol,
        uri,
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };

    let data = create_promo_instruction {
        promo_data,
        metadata_data,
        is_mutable,
        memo,
    }
    .data();

    Ok(Instruction {
        program_id: bpl_token_metadata::id(),
        accounts,
        data,
    })
}

pub fn mint_promo_instruction(
    payer: Pubkey,
    device_owner: Pubkey,
    device: Pubkey,
    location: Pubkey,
    campaign: Pubkey,
    token_owner: Pubkey,
    mint: Pubkey,
    memo: Option<String>,
) -> Result<Instruction, AppError> {
    let authority = find_authority_address().0;
    let promo = find_promo_address(&mint).0;
    let token_account = find_associated_token_address(&token_owner, &mint);
    let campaign_location = find_campaign_location_address(&campaign, &location).0;

    tracing::debug!(
        device_owner = device_owner.to_string(),
        device = device.to_string(),
        lcoation = location.to_string(),
        campaign = campaign.to_string(),
        campaign_location = campaign_location.to_string(),
        token_owner = token_owner.to_string(),
        token_account = token_account.to_string(),
        mint = mint.to_string(),
        memo = memo.clone().unwrap_or("".to_string())
    );

    let accounts = mint_promo_token_accounts {
        payer,
        device_owner,
        device,
        campaign,
        campaign_location,
        token_owner,
        mint,
        authority,
        promo,
        token_account,
        token_program: anchor_spl::token::ID,
        memo_program: spl_memo::ID,
        associated_token_program: anchor_spl::associated_token::ID,
        rent: sysvar::rent::id(),
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let data = mint_promo_token_instruction { memo }.data();

    Ok(Instruction {
        program_id: bpl_token_metadata::id(),
        accounts,
        data,
    })
}

pub fn delegate_promo_instruction(
    payer: Pubkey,
    device_owner: Pubkey,
    device: Pubkey,
    campaign: Pubkey,
    location: Pubkey,
    token_owner: Pubkey,
    mint: Pubkey,
    memo: Option<String>,
) -> Result<Instruction, AppError> {
    let promo = find_promo_address(&mint).0;
    let token_account = find_associated_token_address(&token_owner, &mint);
    let campaign_location = find_campaign_location_address(&campaign, &location).0;

    let accounts = delegate_promo_token_accounts {
        payer,
        device_owner,
        device,
        campaign,
        campaign_location,
        token_owner,
        mint,
        promo,
        token_account,
        memo_program: spl_memo::ID,
        token_program: anchor_spl::token::ID,
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let data = delegate_promo_token_instruction { memo }.data();

    Ok(Instruction {
        program_id: bpl_token_metadata::id(),
        accounts,
        data,
    })
}

pub fn burn_delegated_promo_instruction(
    payer: Pubkey,
    device_owner: Pubkey,
    device: Pubkey,
    location: Pubkey,
    campaign: Pubkey,
    token_account: Pubkey,
    mint: Pubkey,
    platform: Pubkey,
    memo: Option<String>,
) -> Result<Instruction, AppError> {
    let authority = find_authority_address().0;
    let promo = find_promo_address(&mint).0;
    let admin_settings = find_admin_address().0;
    let campaign_location = find_campaign_location_address(&campaign, &location).0;
    // let token_account = find_associated_token_address(&token_owner, &mint);

    let accounts = burn_delegated_promo_token_accounts {
        payer,
        device_owner,
        device,
        campaign,
        campaign_location,
        mint,
        authority,
        promo,
        platform,
        admin_settings,
        token_account,
        memo_program: spl_memo::ID,
        token_program: anchor_spl::token::ID,
        associated_token_program: anchor_spl::associated_token::ID,
        rent: sysvar::rent::id(),
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let data = burn_delegated_promo_token_instruction { memo }.data();

    Ok(Instruction {
        program_id: bpl_token_metadata::id(),
        accounts,
        data,
    })
}

pub fn create_sign_memo_instruction(
    payer: Pubkey,
    memo: String,
    signer: Pubkey,
) -> Result<Instruction, AppError> {
    let accounts = sign_memo_accounts {
        payer,
        signer,
        memo_program: spl_memo::ID,
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let data = sign_memo_instruction { memo }.data();

    Ok(Instruction {
        program_id: bpl_token_metadata::id(),
        accounts,
        data,
    })
}

// Needed to do this since nonblocking client not avaiable in 1.9.20.
pub struct Solana {
    pub cluster: Cluster,
    pub commitment: CommitmentLevel,
    pub client: reqwest::Client,
}

impl Solana {
    pub async fn get_latest_blockhash(&self) -> Result<Hash, AppError> {
        let mut config = serde_json::Map::new();
        config.insert(
            "commitment".to_string(),
            Value::String(self.commitment.to_string()),
        );

        let post_object = PostObject {
            method: String::from("getLatestBlockhash"),
            ..Default::default()
        };

        let result: Value = self
            .client
            .post(self.cluster.url())
            .json(&post_object)
            .send()
            .await?
            .json()
            .await?;

        let hash_str = result["result"]["value"]["blockhash"].as_str().unwrap();
        let hash = Hash::from_str(hash_str)?;
        Ok(hash)
    }

    pub async fn post_transaction(&self, tx_str: &str) -> Result<SendTransResultObject, AppError> {
        let post_object = PostObject {
            params: vec![
                Value::String(tx_str.to_string()),
                json!({"encoding": "base64"}),
            ],
            ..Default::default()
        };

        let response = self
            .client
            .post(self.cluster.url())
            .json(&post_object)
            .send()
            .await
            .map_err(|e| AppError::SolanaPostError(e.to_string()))?;

        let result = response.json::<SendTransResult>().await?;

        tracing::debug!("post_transaction_result {:?}", &result);

        match result {
            SendTransResult::Success(result) => Ok(result),
            SendTransResult::Error(message) => Err(AppError::SolanaPostError(message.message)),
        }
    }

    pub async fn post_transaction_test(&self, tx_str: &str) -> Result<Value, AppError> {
        let mut config = serde_json::Map::new();
        config.insert("encoding".to_string(), Value::String("base64".to_string()));
        config.insert(
            "commitment".to_string(),
            Value::String(self.commitment.to_string()),
        );

        let post_object = PostObject {
            params: vec![Value::String(tx_str.to_string()), Value::Object(config)],
            ..Default::default()
        };

        // let result: SendTransResultObject = self

        let result: Value = self
            .client
            .post(self.cluster.url())
            .json(&post_object)
            .send()
            .await?
            .json()
            .await
            .map_err(|e| AppError::SolanaPostError(e.to_string()))?;

        tracing::debug!("post_transaction_result {:?}", &result);

        Ok(result)
    }

    pub async fn get_transaction(
        &self,
        sig_string: &str,
    ) -> Result<GetTransResultObject, AppError> {
        let mut config = serde_json::Map::new();
        config.insert("encoding".to_string(), Value::String("json".to_string()));
        config.insert(
            "commitment".to_string(),
            Value::String(self.commitment.to_string()),
        );

        let post_object = PostObject {
            method: String::from("getTransaction"),
            params: vec![Value::String(sig_string.to_string()), Value::Object(config)],
            ..Default::default()
        };

        let result: GetTransResultObject = self
            .client
            .post(self.cluster.url())
            .json(&post_object)
            .send()
            .await?
            .json()
            .await
            .map_err(|e| AppError::SolanaGetError(e))?;
        Ok(result)
    }

    pub async fn request_airdrop(&self, pubkey: String, lamports: u64) -> Result<String, AppError> {
        let mut config = serde_json::Map::new();
        config.insert(
            "commitment".to_string(),
            Value::String(self.commitment.to_string()),
        );

        let post_object = PostObject {
            method: "requestAirdrop".to_string(),
            params: vec![json!(pubkey), json!(lamports), Value::Object(config)],
            ..Default::default()
        };

        let result: Value = self
            .client
            .post(self.cluster.url())
            .json(&post_object)
            .send()
            .await?
            .json()
            .await?;

        println!("{}", &result);
        Ok(result["result"].as_str().unwrap().to_string())
    }

    /// Returns wallet balance.
    pub async fn get_balance(&self, address: &Pubkey) -> Result<u64, AppError> {
        let client = reqwest::Client::new();

        let mut config = serde_json::Map::new();
        config.insert("commitment".to_string(), json!("confirmed".to_string()));

        let post_object = PostObject {
            method: String::from("getBalance"),
            params: vec![json!(address.to_string()), Value::Object(config)],
            ..Default::default()
        };

        let result: Value = client
            .post(self.cluster.url())
            .json(&post_object)
            .send()
            .await?
            .json()
            .await?;

        let balance = result["result"]["value"].as_u64().unwrap();
        tracing::debug!(address = address.to_string(), balance = balance);
        Ok(balance)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostObject {
    pub jsonrpc: String,
    pub id: usize,
    pub method: String,
    pub params: Vec<Value>,
}

impl Default for PostObject {
    fn default() -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "sendTransaction".to_string(),
            params: Vec::<Value>::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendTransResultObject {
    pub result: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendTransErrortObject {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SendTransResult {
    Success(SendTransResultObject),
    Error(SendTransErrortObject),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetTransResultObject {
    pub jsonrpc: String,
    pub result: Option<GetTransResultResult>,
    pub block_time: Option<u64>,
    pub id: usize,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Status {
    pub Ok: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTransResultResult {
    pub meta: Meta,
    pub slot: u64,
    pub transaction: Value,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub err: Option<u8>,
    pub fee: u64,
    pub inner_instructions: Vec<u8>,
    pub post_balances: Vec<u64>,
    pub post_token_balances: Vec<u64>,
    pub pre_balances: Vec<u64>,
    pub pre_token_balances: Vec<u64>,
    pub status: Status,
}

// Copied here to avoid depending on anchor client and in turn solana client which was bonking
// the cloud run image because of the usb interface hidapi.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Cluster {
    Testnet,
    Mainnet,
    Devnet,
    Localnet,
    Debug,
    Custom(String, String),
}

impl Default for Cluster {
    fn default() -> Self {
        Cluster::Localnet
    }
}

impl FromStr for Cluster {
    type Err = AppError;
    fn from_str(s: &str) -> Result<Cluster, AppError> {
        match s.to_lowercase().as_str() {
            "t" | "testnet" => Ok(Cluster::Testnet),
            "m" | "mainnet" => Ok(Cluster::Mainnet),
            "d" | "devnet" => Ok(Cluster::Devnet),
            "l" | "localnet" => Ok(Cluster::Localnet),
            "g" | "debug" => Ok(Cluster::Debug),
            _ if s.starts_with("http") => {
                let http_url = s;

                let mut ws_url = url::Url::parse(http_url).map_err(AppError::UrlParseError) ?;
                if let Some(port) = ws_url.port() {
                    ws_url.set_port(Some(port + 1))
                        .map_err(|_| AppError::GenericError("Unable to set port".to_string()))?;
                }
                if ws_url.scheme() == "https" {
                    ws_url.set_scheme("wss")
                        .map_err(|_| AppError::GenericError("Unable to set scheme".to_string()))?;
                } else {
                    ws_url.set_scheme("ws")
                        .map_err(|_| AppError::GenericError("Unable to set scheme".to_string()))?;
                }


                Ok(Cluster::Custom(http_url.to_string(), ws_url.to_string()))
            }
            _ => Err(AppError::GenericError(
                "Cluster must be one of [localnet, testnet, mainnet, devnet] or be an http or https url\n".to_string(),
            )),
        }
    }
}

impl std::fmt::Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let clust_str = match self {
            Cluster::Testnet => "testnet",
            Cluster::Mainnet => "mainnet",
            Cluster::Devnet => "devnet",
            Cluster::Localnet => "localnet",
            Cluster::Debug => "debug",
            Cluster::Custom(url, _ws_url) => url,
        };
        write!(f, "{}", clust_str)
    }
}

impl Cluster {
    pub fn url(&self) -> &str {
        match self {
            Cluster::Devnet => "https://purple-fragrant-dust.solana-mainnet.quiknode.pro/c020b41c62e2d7d6bbee10c7435c85133a0e6bfc",
            Cluster::Testnet => "https://api.testnet.solana.com",
            Cluster::Mainnet => "https://api.mainnet-beta.solana.com",
            Cluster::Localnet => "http://127.0.0.1:8899",
            Cluster::Debug => "http://34.90.18.145:8899",
            Cluster::Custom(url, _ws_url) => url,
        }
    }
    pub fn ws_url(&self) -> &str {
        match self {
            Cluster::Devnet => "wss://purple-fragrant-dust.solana-mainnet.quiknode.pro/c020b41c62e2d7d6bbee10c7435c85133a0e6bfc",
            Cluster::Testnet => "wss://api.testnet.solana.com",
            Cluster::Mainnet => "wss://api.mainnet-beta.solana.com",
            Cluster::Localnet => "ws://127.0.0.1:9000",
            Cluster::Debug => "ws://34.90.18.145:9000",
            Cluster::Custom(_url, ws_url) => ws_url,
        }
    }
}
