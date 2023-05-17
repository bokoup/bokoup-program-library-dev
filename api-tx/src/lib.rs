use axum::{
    error_handling::HandleErrorLayer,
    http::{header, Method, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use bundlr_sdk::{Bundlr, Ed25519Signer};
use ed25519_dalek::SigningKey as DalekKeypair;
use handlers::*;
use solana_sdk::{commitment_config::CommitmentLevel, pubkey::Pubkey, signer::keypair::Keypair};
use std::{borrow::Cow, sync::Arc, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::{
    add_extension::AddExtensionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use url::Url;
use utils::solana::{Cluster, Solana};

pub mod error;
pub mod handlers;
pub mod utils;

// pub const PLATFORM_SIGNER_KEYPAIR_PATH: &str = "/keys/platform_signer/platform_signer-keypair.json";

// `platform` is the address of the account for collecting platform fees
// `platform_signer` is the courtesy signing key that pays minor network
// fees on public mints.
//
// `platform_signer` is also the payer for bundlr transactions.
// TODO: check bundlr balance programatically and alert of running low.

pub fn parse_string_to_keypair(str: &str) -> Keypair {
    let bytes: Vec<u8> = serde_json::from_str(str).unwrap();
    Keypair::from_bytes(&bytes).unwrap()
}

pub struct State {
    pub platform_signer: Keypair,
    pub platform: Pubkey,
    pub solana: Solana,
    pub bundlr: bundlr_sdk::Bundlr<Ed25519Signer>,
    pub data_url: Url,
}

impl State {
    fn new(
        cluster: Cluster,
        commitment: CommitmentLevel,
        platform: Pubkey,
        platform_signer: Keypair,
        data_url: Url,
    ) -> Self {
        let keypair = DalekKeypair::from_bytes(&platform_signer.secret().to_bytes());
        let signer = Ed25519Signer::new(keypair);

        Self {
            platform_signer,
            platform,
            solana: Solana {
                cluster,
                commitment,
                client: reqwest::Client::builder()
                    .timeout(Duration::from_secs(10))
                    .build()
                    .unwrap(),
            },
            bundlr: Bundlr::new(
                "https://node1.bundlr.network".to_string(),
                "solana".to_string(),
                "sol".to_string(),
                signer,
            ),
            data_url,
        }
    }
}

pub fn create_app(
    cluster: Cluster,
    platform: Pubkey,
    platform_signer: Keypair,
    data_url: Url,
) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE])
        .allow_origin(Any);

    Router::new()
        .route(
            "/promo/mint/:mint/:device/:device_owner/:location/:campaign/:message",
            get(get_app_id::handler).post(get_mint_promo_tx::handler),
        )
        .route(
            "/promo/mint/:mint/:device/:device_owner/:location/:campaign/:message/:memo",
            get(get_app_id::handler).post(get_mint_promo_tx::handler),
        )
        .route(
            "/promo/delegate/:mint/:device_owner/:device/:location/:campaign/:message",
            get(get_app_id::handler).post(get_delegate_promo_tx::handler),
        )
        .route(
            "/promo/delegate/:mint/:device_owner/:device/:location/:campaign/:message/:memo",
            get(get_app_id::handler).post(get_delegate_promo_tx::handler),
        )
        .route(
            "/promo/burn-delegated/:mint/:token_account/:device/:location/:campaign/:message",
            get(get_app_id::handler).post(get_burn_delegated_promo_tx::handler),
        )
        .route(
            "/promo/burn-delegated/:mint/:token_account/:device/:location/:campaign/:message/:memo",
            get(get_app_id::handler).post(get_burn_delegated_promo_tx::handler),
        )
        .route(
            "/promo/create/:owner/:campaign",
            get(get_app_id::handler).post(get_create_promo_tx::handler),
        )
        .route(
            "/promo/create/:owner/:campaign/:memo",
            get(get_app_id::handler).post(get_create_promo_tx::handler),
        )
        .route(
            "/signmemo/:message/:memo",
            get(get_app_id::handler).post(get_sign_memo_tx::handler),
        )
        .route(
            "/merchant/create/:owner",
            get(get_app_id::handler).post(get_create_merchant_tx::handler),
        )
        .route(
            "/merchant/create/:owner/:memo",
            get(get_app_id::handler).post(get_create_merchant_tx::handler),
        )
        .route(
            "/location/create/:owner",
            get(get_app_id::handler).post(get_create_location_tx::handler),
        )
        .route(
            "/location/create/:owner/:memo",
            get(get_app_id::handler).post(get_create_location_tx::handler),
        )
        .route(
            "/device/create/:merchant_owner/:location/:owner",
            get(get_app_id::handler).post(get_create_device_tx::handler),
        )
        .route(
            "/device/create/:merchant_owner/:location/:owner/:memo",
            get(get_app_id::handler).post(get_create_device_tx::handler),
        )
        .route(
            "/campaign/create/:owner/:lamports/:memo/*locations",
            get(get_app_id::handler).post(get_create_campaign_tx::handler),
        )
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(30))
                .layer(TraceLayer::new_for_http())
                .layer(AddExtensionLayer::new(Arc::new(State::new(
                    cluster,
                    CommitmentLevel::Confirmed,
                    platform,
                    platform_signer,
                    data_url,
                ))))
                .into_inner(),
        )
}

async fn handle_error(error: BoxError) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
        return (StatusCode::REQUEST_TIMEOUT, Cow::from("request timed out"));
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Cow::from("service is overloaded, try again later"),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Cow::from(format!("Unhandled internal error: {}", error)),
    )
}

// TODO: Move to integration tests - makes live calls to data and transaction apis.
#[cfg(test)]
pub mod test {
    use super::*;
    use anchor_lang::{prelude::Pubkey, AnchorDeserialize};
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use bpl_token_metadata::utils::{find_campaign_address, find_location_address};
    use handlers::PayResponse;
    use solana_sdk::{signature::Signer, transaction::Transaction};
    use std::{
        net::{SocketAddr, TcpListener},
        str::FromStr,
    };
    use tokio::fs;
    use tower::ServiceExt;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    use utils::solana::*;

    const MESSAGE: &str = "This is a really long message that tells you to do something.";
    const PLATFORM: &str = "2R7GkXvQQS4iHptUvQMhDvRSNXL8tAuuASNvCYgz3GQW";
    const DATA_URL: &str = "https://shining-sailfish-15.hasura.app/v1/graphql/";

    #[tokio::test]
    pub async fn run_tests() {
        std::env::set_var("RUST_LOG", "bpl_api_tx=trace");
        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(EnvFilter::from_default_env())
            .init();
        // dotenv::dotenv().ok();
        // fund_accounts().await;
        // test_create_merchant().await;
        // test_create_location().await;
        // test_create_device().await;
    }

    #[tokio::test]
    async fn test_app_id() {
        dotenv::dotenv().ok();
        let platform_signer =
            parse_string_to_keypair(&std::env::var("PLATFORM_SIGNER_KEYPAIR").unwrap());

        // ok to be devnet, only pulling blockhash - will succeed even if localnet validator not running
        let app = create_app(
            Cluster::Devnet,
            Pubkey::from_str(PLATFORM.into()).unwrap(),
            platform_signer,
            Url::from_str(DATA_URL).unwrap(),
        );
        let mint = Pubkey::new_unique();
        let message = urlencoding::encode(MESSAGE);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/promo/mint/{}/{}/{}/{}/{}/{}",
                        mint.to_string(),
                        mint.to_string(),
                        mint.to_string(),
                        mint.to_string(),
                        mint.to_string(),
                        message.into_owned(),
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let parsed_response: get_app_id::ResponseData = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            parsed_response,
            get_app_id::ResponseData {
                label: get_app_id::LABEL.to_string(),
                icon: get_app_id::ICON.to_string(),
            }
        );
    }

    // Testing end user requesting mint tx where merchant has added platform signer to group members
    // to pay for transaction fees with no further merchant approval required.
    #[tokio::test]
    async fn test_get_mint_promo_tx() {
        dotenv::dotenv().ok();
        let platform_signer =
            parse_string_to_keypair(&std::env::var("PLATFORM_SIGNER_KEYPAIR").unwrap());

        let app = create_app(
            Cluster::Devnet,
            Pubkey::from_str(PLATFORM.into()).unwrap(),
            Keypair::from_bytes(&platform_signer.to_bytes()).unwrap(),
            Url::from_str(DATA_URL).unwrap(),
        );

        let mint = Pubkey::new_unique();
        let device = Pubkey::new_unique();
        let location = Pubkey::new_unique();
        let campaign = Pubkey::new_unique();
        let token_owner = Pubkey::new_unique();
        let device_owner = Pubkey::new_unique();

        let data = get_mint_promo_tx::Data {
            account: token_owner.to_string(),
        };
        let message = urlencoding::encode(MESSAGE);
        let memo = "jingus";
        let memo_encoded = urlencoding::encode(memo);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!(
                        "/promo/mint/{}/{}/{}/{}/{}/{}/{}",
                        mint.to_string(),
                        device.to_string(),
                        device_owner.to_string(),
                        location.to_string(),
                        campaign.to_string(),
                        message.into_owned(),
                        memo_encoded.into_owned()
                    ))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&data).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let parsed_response: PayResponse = serde_json::from_slice(&body).unwrap();

        let txd: Transaction = bincode::deserialize(
            &base64::decode::<String>(parsed_response.transaction.clone()).unwrap(),
        )
        .unwrap();

        let instruction = mint_promo_instruction(
            platform_signer.pubkey(),
            device_owner,
            device,
            location,
            campaign,
            token_owner,
            mint,
            Some(memo.to_string()),
        )
        .unwrap();

        let mut tx = Transaction::new_with_payer(&[instruction], Some(&platform_signer.pubkey()));

        let recent_blockhash = txd.message.recent_blockhash;

        tx.try_partial_sign(&[&platform_signer], recent_blockhash)
            .unwrap();

        let serialized = bincode::serialize(&tx).unwrap();
        let transaction = base64::encode(serialized);

        assert_eq!(
            parsed_response,
            PayResponse {
                transaction,
                message: MESSAGE.to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_get_delegate_promo_tx() {
        dotenv::dotenv().ok();
        let platform_signer =
            parse_string_to_keypair(&std::env::var("PLATFORM_SIGNER_KEYPAIR").unwrap());

        let app = create_app(
            Cluster::Devnet,
            Pubkey::from_str(PLATFORM.into()).unwrap(),
            parse_string_to_keypair(&std::env::var("PLATFORM_SIGNER_KEYPAIR").unwrap()),
            Url::from_str(DATA_URL).unwrap(),
        );

        let mint = Pubkey::new_unique();
        let device_owner = Pubkey::new_unique();
        let device = Pubkey::new_unique();
        let campaign = Pubkey::new_unique();
        let location = Pubkey::new_unique();
        let token_owner = Pubkey::new_unique();

        let data = get_mint_promo_tx::Data {
            account: token_owner.to_string(),
        };

        let message = urlencoding::encode(MESSAGE);
        let memo = r#"{"jingus": "amongus"}"#;
        let memo_encoded = urlencoding::encode(memo);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!(
                        "/promo/delegate/{}/{}/{}/{}/{}/{}/{}",
                        mint.to_string(),
                        device_owner.to_string(),
                        device.to_string(),
                        location.to_string(),
                        campaign.to_string(),
                        message.into_owned(),
                        memo_encoded.into_owned()
                    ))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&data).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let parsed_response: PayResponse = serde_json::from_slice(&body).unwrap();

        let txd: Transaction = bincode::deserialize(
            &base64::decode::<String>(parsed_response.transaction.clone()).unwrap(),
        )
        .unwrap();

        let instruction = delegate_promo_instruction(
            platform_signer.pubkey(),
            device_owner,
            device,
            campaign,
            location,
            token_owner,
            mint,
            Some(memo.to_string()),
        )
        .unwrap();

        let mut tx = Transaction::new_with_payer(&[instruction], Some(&platform_signer.pubkey()));
        let recent_blockhash = txd.message.recent_blockhash;

        tx.try_partial_sign(&[&platform_signer], recent_blockhash)
            .unwrap();
        let serialized = bincode::serialize(&tx).unwrap();
        let transaction = base64::encode(serialized);

        assert_eq!(
            parsed_response,
            PayResponse {
                transaction,
                message: MESSAGE.to_owned(),
            }
        );
    }

    #[tokio::test]
    async fn test_get_burn_delegated_promo_tx() {
        dotenv::dotenv().ok();
        let platform_signer =
            parse_string_to_keypair(&std::env::var("PLATFORM_SIGNER_KEYPAIR").unwrap());

        let app = create_app(
            Cluster::Devnet,
            Pubkey::from_str(PLATFORM.into()).unwrap(),
            parse_string_to_keypair(&std::env::var("PLATFORM_SIGNER_KEYPAIR").unwrap()),
            Url::from_str(DATA_URL).unwrap(),
        );

        let mint = Pubkey::new_unique();
        let token_account = Pubkey::new_unique();
        let device = Pubkey::new_unique();
        let campaign = Pubkey::new_unique();
        let location = Pubkey::new_unique();
        let device_owner = Pubkey::new_unique();

        let data = get_mint_promo_tx::Data {
            account: device_owner.to_string(),
        };

        let message = urlencoding::encode(MESSAGE);
        let memo = r#"{"jingus": "amongus"}"#;
        let memo_encoded = urlencoding::encode(memo);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!(
                        "/promo/burn-delegated/{}/{}/{}/{}/{}/{}/{}",
                        mint.to_string(),
                        token_account.to_string(),
                        device.to_string(),
                        location.to_string(),
                        campaign.to_string(),
                        message.into_owned(),
                        memo_encoded.into_owned()
                    ))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&data).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let parsed_response: PayResponse = serde_json::from_slice(&body).unwrap();

        let txd: Transaction = bincode::deserialize(
            &base64::decode::<String>(parsed_response.transaction.clone()).unwrap(),
        )
        .unwrap();

        let instruction = burn_delegated_promo_instruction(
            platform_signer.pubkey(),
            device_owner,
            device,
            location,
            campaign,
            token_account,
            mint,
            Pubkey::from_str(PLATFORM.into()).unwrap(),
            Some(memo.to_string()),
        )
        .unwrap();

        let mut tx = Transaction::new_with_payer(&[instruction], Some(&platform_signer.pubkey()));
        let recent_blockhash = txd.message.recent_blockhash;

        tx.try_partial_sign(&[&platform_signer], recent_blockhash)
            .unwrap();
        let serialized = bincode::serialize(&tx).unwrap();
        let transaction = base64::encode(serialized);

        assert_eq!(
            parsed_response,
            PayResponse {
                transaction,
                message: MESSAGE.to_owned(),
            }
        );
    }

    #[tokio::test]
    async fn test_sign_memo_tx() {
        dotenv::dotenv().ok();
        let platform_signer =
            parse_string_to_keypair(&std::env::var("PLATFORM_SIGNER_KEYPAIR").unwrap());

        let signer = Keypair::new();

        let state = State::new(
            Cluster::Localnet,
            CommitmentLevel::Confirmed,
            Pubkey::from_str(PLATFORM.into()).unwrap(),
            Keypair::from_bytes(&platform_signer.to_bytes()).unwrap(),
            Url::from_str(DATA_URL).unwrap(),
        );
        // ok to be devnet, only pulling blockhash - will succeed even if localnet validator not running
        let app = create_app(
            Cluster::Devnet,
            Pubkey::from_str(PLATFORM.into()).unwrap(),
            platform_signer,
            Url::from_str(DATA_URL).unwrap(),
        );

        let pre_memo = r#"{"jingus": "amongus"}"#;
        let memo = urlencoding::encode(pre_memo);

        let data = get_mint_promo_tx::Data {
            account: signer.pubkey().to_string(),
        };

        let message = urlencoding::encode(MESSAGE);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!(
                        "/signmemo/{}/{}",
                        message.into_owned(),
                        memo.to_string(),
                    ))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&data).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let parsed_response: PayResponse = serde_json::from_slice(&body).unwrap();

        let txd: Transaction = bincode::deserialize(
            &base64::decode::<String>(parsed_response.transaction.clone()).unwrap(),
        )
        .unwrap();

        let instruction = create_sign_memo_instruction(
            state.platform_signer.pubkey(),
            pre_memo.to_string(),
            signer.pubkey(),
        )
        .unwrap();

        let mut tx =
            Transaction::new_with_payer(&[instruction], Some(&state.platform_signer.pubkey()));
        let recent_blockhash = txd.message.recent_blockhash;

        tx.try_partial_sign(&[&state.platform_signer], recent_blockhash)
            .unwrap();
        let serialized = bincode::serialize(&tx).unwrap();
        let transaction = base64::encode(serialized);

        assert_eq!(
            parsed_response,
            PayResponse {
                transaction,
                message: MESSAGE.to_owned(),
            }
        );
    }

    #[tokio::test]
    async fn test_create_merchant() {
        dotenv::dotenv().ok();
        let merchant_owner =
            parse_string_to_keypair(&std::env::var("MERCHANT_OWNER_KEYPAIR").unwrap());
        let platform_signer =
            parse_string_to_keypair(&std::env::var("PLATFORM_SIGNER_KEYPAIR").unwrap());

        let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(
                    create_app(
                        Cluster::Devnet,
                        Pubkey::from_str(PLATFORM.into()).unwrap(),
                        platform_signer,
                        Url::from_str(DATA_URL).unwrap(),
                    )
                    .into_make_service(),
                )
                .await
                .unwrap();
        });

        let file_path = "./tests/fixtures/bokoup_logo_3.jpg";
        let file = fs::read(file_path).await.unwrap();

        let content_type = if let Some(content_type) = mime_guess::from_path(file_path).first() {
            content_type.to_string()
        } else {
            mime_guess::mime::OCTET_STREAM.to_string()
        };

        let metadata_data = serde_json::json!({
            "name": "Test Merchant",
            "website": "https://bokoup.dev",
            "description": "bokout test merchant",
            "active": true
        });

        let form = reqwest::multipart::Form::new()
            .part(
                "metadata",
                reqwest::multipart::Part::text(metadata_data.to_string())
                    .mime_str("application/json")
                    .unwrap(),
            )
            .part(
                "image",
                reqwest::multipart::Part::bytes(file)
                    .file_name(file_path.split("/").last().unwrap())
                    .mime_str(&content_type)
                    .unwrap(),
            );

        let memo =
            serde_json::json!({"reference": "tester", "memo": "have a great day"}).to_string();
        let client = reqwest::Client::new();

        tracing::debug!(merchant_owner = merchant_owner.pubkey().to_string());

        let response = client
            .post(format!(
                "http://{}/merchant/create/{}/{}",
                addr,
                merchant_owner.pubkey(),
                memo
            ))
            .multipart(form)
            .send()
            .await
            .unwrap()
            .json::<PayResponse>()
            .await
            .unwrap();

        let tx: Transaction =
            bincode::deserialize(&base64::decode::<String>(response.transaction.clone()).unwrap())
                .unwrap();

        let instruction = bpl_token_metadata::instruction::CreateMerchant::try_from_slice(
            &tx.message.instructions[0].data[8..],
        )
        .unwrap();

        assert_eq!(instruction.data.name, "Test Merchant".to_string());
    }

    #[tokio::test]
    async fn test_create_location() {
        dotenv::dotenv().ok();
        let merchant_owner =
            parse_string_to_keypair(&std::env::var("MERCHANT_OWNER_KEYPAIR").unwrap());
        let platform_signer =
            parse_string_to_keypair(&std::env::var("PLATFORM_SIGNER_KEYPAIR").unwrap());

        let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(
                    create_app(
                        Cluster::Devnet,
                        Pubkey::from_str(PLATFORM.into()).unwrap(),
                        platform_signer,
                        Url::from_str(DATA_URL).unwrap(),
                    )
                    .into_make_service(),
                )
                .await
                .unwrap();
        });

        let file_path = "./tests/fixtures/bokoup_logo_3.jpg";
        let file = fs::read(file_path).await.unwrap();

        let content_type = if let Some(content_type) = mime_guess::from_path(file_path).first() {
            content_type.to_string()
        } else {
            mime_guess::mime::OCTET_STREAM.to_string()
        };

        let metadata_data = serde_json::json!({
            "name": "Test Location",
            "website": "https://bokoup.dev",
            "description": "bokout test location",
            "address": "123 Main Street, Anytown, CA 12345",
            "active": true
        });

        let form = reqwest::multipart::Form::new()
            .part(
                "metadata",
                reqwest::multipart::Part::text(metadata_data.to_string())
                    .mime_str("application/json")
                    .unwrap(),
            )
            .part(
                "image",
                reqwest::multipart::Part::bytes(file)
                    .file_name(file_path.split("/").last().unwrap())
                    .mime_str(&content_type)
                    .unwrap(),
            );

        let memo =
            serde_json::json!({"reference": "tester", "memo": "have a great day"}).to_string();
        let client = reqwest::Client::new();

        let response = client
            .post(format!(
                "http://{}/location/create/{}/{}",
                addr,
                merchant_owner.pubkey(),
                memo,
            ))
            .multipart(form)
            .send()
            .await
            .unwrap()
            .json::<PayResponse>()
            .await
            .unwrap();

        let tx: Transaction =
            bincode::deserialize(&base64::decode::<String>(response.transaction.clone()).unwrap())
                .unwrap();

        let instruction = bpl_token_metadata::instruction::CreateLocation::try_from_slice(
            &tx.message.instructions[0].data[8..],
        )
        .unwrap();

        assert_eq!(instruction.data.name, "Test Location".to_string());
    }

    #[tokio::test]
    async fn test_create_device() {
        dotenv::dotenv().ok();
        let merchant_owner =
            parse_string_to_keypair(&std::env::var("MERCHANT_OWNER_KEYPAIR").unwrap());
        let platform_signer =
            parse_string_to_keypair(&std::env::var("PLATFORM_SIGNER_KEYPAIR").unwrap());

        let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(
                    create_app(
                        Cluster::Devnet,
                        Pubkey::from_str(PLATFORM.into()).unwrap(),
                        platform_signer,
                        Url::from_str(DATA_URL).unwrap(),
                    )
                    .into_make_service(),
                )
                .await
                .unwrap();
        });

        let metadata_data = serde_json::json!({
            "name": "Test Device",
            "reference": "012345677",
            "description": "bokout test location",
            "active": true
        });

        let form = reqwest::multipart::Form::new().part(
            "metadata",
            reqwest::multipart::Part::text(metadata_data.to_string())
                .mime_str("application/json")
                .unwrap(),
        );

        let location = find_location_address(&merchant_owner.pubkey(), "Test Location").0;

        let memo =
            serde_json::json!({"reference": "tester", "memo": "have a great day"}).to_string();
        let client = reqwest::Client::new();

        let response = client
            .post(format!(
                "http://{}/device/create/{}/{}/{}/{}",
                addr,
                merchant_owner.pubkey(),
                location,
                Pubkey::new_unique(),
                memo,
            ))
            .multipart(form)
            .send()
            .await
            .unwrap()
            .json::<PayResponse>()
            .await
            .unwrap();

        let tx: Transaction =
            bincode::deserialize(&base64::decode::<String>(response.transaction.clone()).unwrap())
                .unwrap();

        let instruction = bpl_token_metadata::instruction::CreateDevice::try_from_slice(
            &tx.message.instructions[0].data[8..],
        )
        .unwrap();

        assert_eq!(instruction.data.name, "Test Device".to_string());
    }

    #[tokio::test]
    async fn test_create_campaign() {
        dotenv::dotenv().ok();
        let merchant_owner =
            parse_string_to_keypair(&std::env::var("MERCHANT_OWNER_KEYPAIR").unwrap());
        let platform_signer =
            parse_string_to_keypair(&std::env::var("PLATFORM_SIGNER_KEYPAIR").unwrap());

        let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(
                    create_app(
                        Cluster::Devnet,
                        Pubkey::from_str(PLATFORM.into()).unwrap(),
                        platform_signer,
                        Url::from_str(DATA_URL).unwrap(),
                    )
                    .into_make_service(),
                )
                .await
                .unwrap();
        });

        let metadata_data = serde_json::json!({
            "name": "Test Campaign",
            "reference": "012345677",
            "description": "bokout test location",
            "active": true
        });

        let form = reqwest::multipart::Form::new().part(
            "metadata",
            reqwest::multipart::Part::text(metadata_data.to_string())
                .mime_str("application/json")
                .unwrap(),
        );

        let location = find_location_address(&merchant_owner.pubkey(), "Test Location").0;
        let location2 = find_location_address(&merchant_owner.pubkey(), "Test Location 2").0;

        let memo =
            serde_json::json!({"reference": "tester", "memo": "have a great day"}).to_string();
        let client = reqwest::Client::new();

        let response = client
            .post(format!(
                "http://{}/campaign/create/{}/{}/{}/{}/{}",
                addr,
                merchant_owner.pubkey(),
                1_000_000_000,
                memo,
                location,
                location2
            ))
            .multipart(form)
            .send()
            .await
            .unwrap()
            .json::<PayResponse>()
            .await
            .unwrap();

        let tx: Transaction =
            bincode::deserialize(&base64::decode::<String>(response.transaction.clone()).unwrap())
                .unwrap();

        let instruction = bpl_token_metadata::instruction::CreateCampaign::try_from_slice(
            &tx.message.instructions[0].data[8..],
        )
        .unwrap();

        assert_eq!(instruction.data.name, "Test Campaign".to_string());
    }

    #[tokio::test]
    async fn test_create_promo() {
        dotenv::dotenv().ok();
        let merchant_owner =
            parse_string_to_keypair(&std::env::var("MERCHANT_OWNER_KEYPAIR").unwrap());
        let platform_signer =
            parse_string_to_keypair(&std::env::var("PLATFORM_SIGNER_KEYPAIR").unwrap());

        let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();

        // ok to be devnet, just getting blockhash for test
        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(
                    create_app(
                        Cluster::Devnet,
                        Pubkey::from_str(PLATFORM.into()).unwrap(),
                        platform_signer,
                        Url::from_str(DATA_URL).unwrap(),
                    )
                    .into_make_service(),
                )
                .await
                .unwrap();
        });

        let file_path = "./tests/fixtures/bokoup_logo_3.jpg";
        let file = fs::read(file_path).await.unwrap();

        let content_type = if let Some(content_type) = mime_guess::from_path(file_path).first() {
            content_type.to_string()
        } else {
            mime_guess::mime::OCTET_STREAM.to_string()
        };

        let metadata_data = serde_json::json!({
            "name": "buyXProduct",
            "symbol": "PROD",
            "description": "bokoup test promo - product",
            "attributes": [
                {
                    "trait_type": "promoType",
                    "value": "buyXProductGetYFree",
                },
                {
                    "trait_type": "productId",
                    "value": "0E9DCHTY6P7M2",
                },
                {
                    "trait_type": "buyXProduct",
                    "value": 3
                },
                {
                    "trait_type": "getYProduct",
                    "value": 1
                },
                {  "trait_type": "maxMint",
                    "value": 1000,
                },
                {
                    "trait_type": "maxBurn",
                    "value": 500,
                },
            ],
            "collection": {
                "name": "Product Promo",
                "family": "Test Merchant Promos"
            },
            "active": true
        });

        let form = reqwest::multipart::Form::new()
            .part(
                "metadata",
                reqwest::multipart::Part::text(metadata_data.to_string())
                    .mime_str("application/json")
                    .unwrap(),
            )
            .part(
                "image",
                reqwest::multipart::Part::bytes(file)
                    .file_name(file_path.split("/").last().unwrap())
                    .mime_str(&content_type)
                    .unwrap(),
            );

        let campaign = find_campaign_address(&merchant_owner.pubkey(), "Test Campaign").0;

        let memo =
            serde_json::json!({"reference": "tester", "memo": "have a great day"}).to_string();
        let client = reqwest::Client::new();

        let response = client
            .post(format!(
                "http://{}/promo/create/{}/{}/{}",
                addr,
                merchant_owner.pubkey(),
                campaign,
                memo,
            ))
            .multipart(form)
            .send()
            .await
            .unwrap()
            .json::<PayResponse>()
            .await
            .unwrap();

        let tx: Transaction =
            bincode::deserialize(&base64::decode::<String>(response.transaction.clone()).unwrap())
                .unwrap();

        let instruction = bpl_token_metadata::instruction::CreatePromo::try_from_slice(
            &tx.message.instructions[0].data[8..],
        )
        .unwrap();

        assert_eq!(instruction.metadata_data.name, "buyXProduct".to_string());
    }
}
