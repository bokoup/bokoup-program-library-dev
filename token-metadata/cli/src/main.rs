use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        signature::{read_keypair_file, Keypair},
        signer::Signer,
        system_program
    },
    Client, Cluster,
};
use bpl_token_metadata::{instruction, accounts, state::{AdminSettings, Campaign}, utils::{self, find_campaign_address, find_merchant_address}};
use bundlr_sdk::{tags::Tag, Bundlr, Ed25519Signer};
use clap::{Parser, Subcommand, ArgEnum};
use ed25519_dalek::SigningKey as DalekKeypair;
use tokio::time::sleep;
use std::{path::PathBuf, rc::Rc, time::Duration};
use tracing_subscriber::prelude::*;

#[derive(ArgEnum, Clone, Debug)]
enum Address {
    ProgramAuthority,
    Platform,
    PlatformSigner,
    Merchant,
    Campaign,
    DeviceOnwer
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    #[clap(short, long)]
    quiet: bool,
    #[clap(long, default_value = "/home/caleb/.config/solana/devnet.json", value_parser = valid_file_path)]
    program_authority_path: PathBuf,
    #[clap(long, default_value = "../../target/deploy/platform-keypair.json", value_parser = valid_file_path)]
    platform_path: PathBuf,
    #[clap(long, default_value = "../../target/deploy/platform_signer-keypair.json", value_parser = valid_file_path)]
    platform_signer_path: PathBuf,
    #[clap(long, default_value = "../../target/deploy/merchant-keypair.json", value_parser = valid_file_path)]
    merchant_path: PathBuf,
    #[clap(long, default_value = "../../target/deploy/device_owner-keypair.json", value_parser = valid_file_path)]
    device_owner_path: PathBuf,
    #[clap(long, default_value_t = Cluster::Localnet, value_parser)]
    cluster: Cluster,
}

#[derive(Subcommand)]
enum Commands {
    Airdrop {
        #[clap(default_value_t = 2, value_parser)]
        sol: u64,
    },
    #[clap(about = "Placeholder demonstrating upload of json to arweave")]
    UploadString,
    #[clap(about = "Create or update admin settings account")]
    CreateAdminSettings {
        #[clap(long, default_value_t = 100_000_000, value_parser)]
        create_promo_lamports: u64,
        #[clap(long, default_value_t = 10_000_000, value_parser)]
        burn_promo_token_lamports: u64,
    },
    CreateCampaign {
        #[clap(long, default_value = "Test Campaign")]
        name: String,
        #[clap(long, default_value = "https://campaign.example.com")]
        uri: String,
        #[clap(long, default_value_t = 500_000_000, value_parser)]
        lamports: u64,
    },
    #[clap(about = "Tesing requesting data from graphql api")]
    TestGql,
    Balance {
        #[clap(arg_enum, default_value = "campaign")]
        address: Address,
        #[clap(long, default_value = "Test Campaign")]
        campaign_name: String
    },
    InstructionDiscriminator {
        #[clap(index=1)]
        name: String
    },
    PurgeImgx {
        #[clap(index=1)]
        url: String
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let cli = Cli::parse();

    if !cli.quiet {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "bpl_token_metadata_cli=trace".into()),
            ))
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    let [
        program_authority_keypair,
        platform_keypair,
        platform_signer_keypair,
        merchant_keypair,
        device_owner_keypair,
        ]: [Keypair; 5] = 
        [
        &cli.program_authority_path,
        &cli.platform_path,
        &cli.platform_signer_path,
        &cli.merchant_path,
        &cli.device_owner_path,
    ]
    .map(|p| read_keypair_file(p).expect("problem reading keypair file"));

    match &cli.command {
        Commands::Airdrop { sol } => {
            let program_authority = program_authority_keypair.pubkey();
            let rc_payer_keypair = Rc::new(program_authority_keypair);
            let client = Client::new_with_options(
                cli.cluster,
                rc_payer_keypair,
                CommitmentConfig::confirmed(),
            );

            let program = client.program(bpl_token_metadata::id());
            
            for pubkey in [
                &program_authority,
                &platform_signer_keypair.pubkey(),
                &merchant_keypair.pubkey(),
            ] {
                program
                .rpc()
                .request_airdrop(pubkey, sol * 1_000_000_000)
                .unwrap();
                sleep(Duration::from_secs(1)).await;
            };
            Ok(())
        }
        Commands::Balance { address, campaign_name } => {
            let program_authority = program_authority_keypair.pubkey();
            let rc_payer_keypair = Rc::new(program_authority_keypair);
            let client = Client::new_with_options(
                cli.cluster,
                rc_payer_keypair,
                CommitmentConfig::confirmed(),
            );

            let program = client.program(bpl_token_metadata::id());
            
            let pubkey = match address {
                Address::ProgramAuthority => {
                    program_authority
                }
                Address::Platform => {
                    platform_keypair.pubkey()
                }
                Address::PlatformSigner => {
                    platform_signer_keypair.pubkey()
                }
                Address::Merchant => {
                    merchant_keypair.pubkey()
                }
                Address::Campaign => {
                    let (address, _) = find_campaign_address(&merchant_keypair.pubkey(), campaign_name);
                    address
                }
                Address::DeviceOnwer => {
                    device_owner_keypair.pubkey()
                }
            };
            let balance = program
                .rpc()
                .get_balance(&pubkey)
                .unwrap();

            println!("{:?}: {{address: {pubkey}, balance: {balance}}}", address);
            Ok(())
        }
        Commands::CreateAdminSettings {
            create_promo_lamports,
            burn_promo_token_lamports,
        } => {
            let payer = program_authority_keypair.pubkey();
            let rc_payer_keypair = Rc::new(program_authority_keypair);
            let client = Client::new_with_options(
                cli.cluster,
                rc_payer_keypair,
                CommitmentConfig::confirmed(),
            );

            let program = client.program(bpl_token_metadata::id());
            let (admin_settings, _) = utils::find_admin_address();
            
            let program_data = utils::find_program_data_address();
            tracing::info!(program_data = program_data.to_string());

            let tx = program
                .request()
                .accounts(bpl_token_metadata::accounts::CreateAdminSettings {
                    payer,
                    admin_settings,
                    // program: bpl_token_metadata::id(),
                    // program_data,
                    system_program: system_program::ID,
                })
                .args(instruction::CreateAdminSettings {
                    data: AdminSettings {
                        platform: platform_keypair.pubkey(),
                        create_promo_lamports: create_promo_lamports.clone(),
                        burn_promo_token_lamports: burn_promo_token_lamports.clone(),
                    },
                })
                .send()?;
            let admin_settings_account: AdminSettings = program.account(admin_settings)?;
            tracing::info!(
                signature = tx.to_string(),
                admin_settings_account = format!("{:?}", admin_settings_account)
            );
            Ok(())
        }
        Commands::CreateCampaign {
            name,
            uri,
            lamports
        } => {
            let owner = merchant_keypair.pubkey();
            let payer = platform_signer_keypair.pubkey();
            let rc_payer_keypair = Rc::new(merchant_keypair);
            let client = Client::new_with_options(
                cli.cluster,
                rc_payer_keypair,
                CommitmentConfig::confirmed(),
            );

            let program = client.program(bpl_token_metadata::id());

            let merchant = find_merchant_address(&owner).0;
            let campaign = find_campaign_address(&merchant, name).0;
            
            // this needs to be updated for actual location, just addresses for now
            let data = Campaign {
                    merchant,
                    name: name.clone(),
                    uri: uri.clone(),
                    active: true,
                };

            let tx = program
            .request()
            .accounts(accounts::CreateCampaign {
                payer,
                owner,
                merchant,
                campaign,
                memo_program: spl_memo::ID,
                system_program: system_program::ID,
            })
            .args(instruction::CreateCampaign {
                data,
                lamports: lamports.clone(),
                memo: None,
            })
            .send()?;
            
            let campaign_account: Campaign = program.account(campaign)?;
            tracing::info!(
                signature = tx.to_string(),
                campaign = format!("{:?}", campaign_account)
            );
                
            Ok(())

        }
        Commands::UploadString => {
            let mut data = tokio::fs::read(&cli.program_authority_path).await.unwrap();
            let mut bytes = [0_u8; 64];
            data.iter_mut().enumerate().for_each(|(i, v)| bytes[i] = *v);
            let keypair = DalekKeypair::from_keypair_bytes(&bytes).unwrap();
            tracing::debug!(signer = bs58::encode(&keypair.verifying_key().as_ref()).into_string());
            let signer = Ed25519Signer::new(keypair);

            let bundlr = Bundlr::new(
                "https://node1.bundlr.network".to_string(),
                "solana".to_string(),
                "sol".to_string(),
                signer,
            );

            let json_data = serde_json::json!({
                "name": "Test Promo",
                "symbol": "TEST",
                "description": "Bokoup test promotion.",
                "attributes": [
                    {  "trait_type": "discount",
                        "value": 10,
                    },
                    {
                        "trait_type": "expiration",
                        "value": "never",
                    },
                ],
                "collection": {
                    "name": "Test Merchant Promos",
                    "family": "Non-Fungible Offers"
                },
                "max_mint": 1000,
                "max_burn": 500
            });

            let tx = bundlr.create_transaction_with_tags(
                serde_json::to_vec(&json_data).unwrap(),
                vec![
                    Tag::new("User-Agent".into(), "bokoup".into()),
                    Tag::new("Content-Type".into(), "application/json".into()),
                ],
            );

            // Will return Err if not success
            match bundlr.send_transaction(tx).await {
                Ok(value) => println!("{}", value),
                Err(e) => println!("{}", e),
            }
            Ok(())
        }
        Commands::TestGql => {
            let client = reqwest::Client::new();
            let query = r#"
            query MintQuery($mint: String) {
                mint(where: {id: {_eq: $mint}}) {
                  promoObject {
                    groupObject {
                      id
                      seed
                    }
                  }
                }
              }
            "#;

            let result: serde_json::Value = client
                .post("https://shining-sailfish-15.hasura.app/v1/graphql/")
                .json(&serde_json::json!({ "query": query, "operationName": "MintQuery", "variables": {"mint": "4uQeVj5tqViQh7yWWGStvkEG1Zmhx6uasJtWCJziofM"}}))
                .send()
                .await?
                .json()
                .await?;

            println!("{}", result);
            Ok(())
        }
        Commands::InstructionDiscriminator { name } => {
            pub fn sighash(namespace: &str, name: &str) {
                let preimage = format!("{}:{}", namespace, name);
            
                let mut sighash = [0u8; 8];
                sighash.copy_from_slice(
                    &anchor_client::anchor_lang::solana_program::hash::hash(preimage.as_bytes()).to_bytes()
                        [..8],
                );
                println!("{:?}", sighash);

            }

            sighash("global", name); 
            
            Ok(())
        }
        Commands::PurgeImgx { url } => {
            let client = reqwest::Client::new();

            let body = serde_json::json!(
                {
                    "data": {
                        "attributes": {
                            "url": &url
                        },
                        "type": "purges"
                    }
                }
            );

            let result = client.post("https://api.imgix.com/api/v1/purge")
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", std::env::var("IMGIX_API_TOKEN").unwrap()))
            .json(&body)
            .send().await?;

            println!("{}", result.text().await?);

            Ok(())
        }

    }
}

// https://docs.bundlr.network/docs/client/examples/funding-your-account

// ====================
// Validators
// ====================

fn valid_file_path(path_str: &str) -> Result<PathBuf, String> {
    match path_str.parse::<PathBuf>() {
        Ok(p) => {
            if p.exists() {
                if p.is_file() {
                    Ok(p)
                } else {
                    Err(format!("path is not file."))
                }
            } else {
                Err(format!("path does not exist."))
            }
        }
        Err(_) => Err(format!("not a valid path.")),
    }
}
