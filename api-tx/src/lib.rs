use axum::{
    error_handling::HandleErrorLayer,
    http::{header, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use bundlr_sdk::{Bundlr, Ed25519Signer};
use ed25519_dalek::Keypair as DalekKeypair;
use handlers::*;
use solana_sdk::{
    commitment_config::CommitmentLevel, pubkey::Pubkey, signature::read_keypair_file,
    signer::keypair::Keypair,
};
use std::{borrow::Cow, str::FromStr, sync::Arc, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::{
    add_extension::AddExtensionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use url::Url;
use utils::{
    clover::Clover,
    solana::{Cluster, Solana},
};

pub mod error;
pub mod handlers;
pub mod utils;

pub const CLOVER_URL: &str = "https://sandbox.dev.clover.com/v3/apps/";
pub const CLOVER_APP_ID: &str = "MAC8DQKWCCB1R";
pub const PROMO_OWNER_KEYPAIR_PATH: &str = "/keys/promo_owner-keypair.json";

pub struct State {
    pub promo_owner: Keypair,
    pub platform: Pubkey,
    pub solana: Solana,
    pub clover: Clover,
    pub bundlr: bundlr_sdk::Bundlr<Ed25519Signer>,
    pub data_url: String,
}

impl State {
    fn new(cluster: Cluster) -> Self {
        let data = std::fs::read(PROMO_OWNER_KEYPAIR_PATH).unwrap();
        let bytes: Vec<u8> = serde_json::from_slice(&data).unwrap();
        let keypair = DalekKeypair::from_bytes(&bytes).unwrap();
        let signer = Ed25519Signer::new(keypair);

        Self {
            promo_owner: read_keypair_file(PROMO_OWNER_KEYPAIR_PATH).unwrap(),
            platform: Pubkey::from_str("2R7GkXvQQS4iHptUvQMhDvRSNXL8tAuuASNvCYgz3GQW").unwrap(),
            solana: Solana {
                cluster,
                commitment: CommitmentLevel::Confirmed,
                client: reqwest::Client::builder()
                    .timeout(Duration::from_secs(10))
                    .build()
                    .unwrap(),
            },
            clover: Clover {
                base_url: CLOVER_URL
                    .parse::<Url>()
                    .unwrap()
                    .join(format!("{CLOVER_APP_ID}/").as_str())
                    .unwrap(),
                client: reqwest::Client::new(),
            },
            bundlr: Bundlr::new(
                "https://node1.bundlr.network".to_string(),
                "solana".to_string(),
                "sol".to_string(),
                signer,
            ),
            data_url: "https://shining-sailfish-15.hasura.app/v1/graphql/".to_string(),
        }
    }
}

pub fn create_app(cluster: Cluster) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE])
        .allow_origin(Any);

    Router::new()
        .route(
            "/promo/mint/:mint_string/:message",
            get(get_app_id::handler).post(get_mint_promo_tx::handler),
        )
        .route(
            "/promo/mint/:mint_string/:message/:memo",
            get(get_app_id::handler).post(get_mint_promo_tx::handler),
        )
        .route(
            "/promo/delegate/:mint_string/:message",
            get(get_app_id::handler).post(get_delegate_promo_tx::handler),
        )
        .route(
            "/promo/delegate/:mint_string/:message/:memo",
            get(get_app_id::handler).post(get_delegate_promo_tx::handler),
        )
        .route(
            "/promo/burn-delegated/:mint_string/:message",
            get(get_app_id::handler).post(burn_delegated_promo::handler),
        )
        .route(
            "/promo/burn-delegated/:mint_string/:message/:memo",
            get(get_app_id::handler).post(burn_delegated_promo::handler),
        )
        .route(
            "/promo/create",
            post(create_promo::handler), // .layer(DefaultBodyLimit::disable())
                                         // .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024 /* 10mb */)), // ),
        )
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(30))
                .layer(TraceLayer::new_for_http())
                .layer(AddExtensionLayer::new(Arc::new(State::new(cluster))))
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
    use anchor_lang::prelude::Pubkey;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use serde_json::Value;
    use solana_sdk::{signature::Signer, transaction::Transaction};
    use std::net::{SocketAddr, TcpListener};
    use tokio::fs;
    use tower::ServiceExt;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    use utils::{data::*, solana::*};

    const MESSAGE: &str = "This is a really long message that tells you to do something.";

    async fn fetch_mint(url: &String) -> Value {
        let client = reqwest::Client::new();
        let result: serde_json::Value = client
            .post(url)
            .json(&serde_json::json!({ "query": MINT_QUERY }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        tracing::debug!(result = result.to_string());

        let mint = result
            .as_object()
            .unwrap()
            .get("data")
            .unwrap()
            .as_object()
            .unwrap()
            .get("mint")
            .unwrap()
            .as_array()
            .unwrap()
            .get(0)
            .unwrap()
            .as_object()
            .unwrap()
            .get("id")
            .unwrap()
            .as_str()
            .unwrap();
        serde_json::json!({ "mint": mint })
    }

    #[tokio::test]
    async fn test_app_id() {
        std::env::set_var("RUST_LOG", "bpl_api_tx=trace");
        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(EnvFilter::from_default_env())
            .init();

        // ok to be devnet, only pulling blockhash - will succeed even if localnet validator not running
        let app = create_app(Cluster::Devnet);
        let mint = Pubkey::new_unique();
        let message = urlencoding::encode(MESSAGE);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/promo/mint/{}/{}",
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

    #[tokio::test]
    async fn test_get_mint_promo_tx() {
        // ok to be devnet, only pulling blockhash - will succeed even if localnet validator not running
        let state = State::new(Cluster::Devnet);
        let app = create_app(Cluster::Devnet);
        let token_owner = Pubkey::new_unique();

        let result = fetch_mint(&state.data_url).await;

        let mint_str = result
            .as_object()
            .unwrap()
            .get("mint")
            .unwrap()
            .as_str()
            .unwrap();

        let mint = Pubkey::from_str(mint_str).unwrap();

        let query = serde_json::json!({ "query": PROMO_GROUP_QUERY, "variables": {"mint": mint.to_string()}});
        let result: serde_json::Value = state
            .solana
            .client
            .post(&state.data_url)
            .json(&query)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let group = get_group(&state.promo_owner.pubkey(), &result).unwrap();

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
                        "/promo/mint/{}/{}/{}",
                        mint.to_string(),
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
        let parsed_response: get_mint_promo_tx::ResponseData =
            serde_json::from_slice(&body).unwrap();

        let txd: Transaction = bincode::deserialize(
            &base64::decode::<String>(parsed_response.transaction.clone()).unwrap(),
        )
        .unwrap();

        let instruction = create_mint_promo_instruction(
            state.promo_owner.pubkey(),
            group,
            token_owner,
            mint,
            Some(memo.to_string()),
        )
        .unwrap();

        let mut tx = Transaction::new_with_payer(&[instruction], Some(&state.promo_owner.pubkey()));
        tx.try_partial_sign(&[&state.promo_owner], txd.message.recent_blockhash)
            .unwrap();
        let serialized = bincode::serialize(&tx).unwrap();
        let transaction = base64::encode(serialized);

        assert_eq!(
            parsed_response,
            get_mint_promo_tx::ResponseData {
                transaction,
                message: MESSAGE.to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_get_delegate_promo_tx() {
        dotenv::dotenv().ok();

        // ok to be devnet, only pulling blockhash - will succeed even if localnet validator not running
        let state = State::new(Cluster::Devnet);
        let app = create_app(Cluster::Devnet);
        let token_owner = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let group_seed = Pubkey::new_unique();

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
                        "/promo/delegate/{}/{}/{}",
                        mint.to_string(),
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
        let parsed_response: burn_delegated_promo::ResponseData =
            serde_json::from_slice(&body).unwrap();

        let txd: Transaction = bincode::deserialize(
            &base64::decode::<String>(parsed_response.transaction.clone()).unwrap(),
        )
        .unwrap();

        let instruction = create_delegate_promo_instruction(
            state.promo_owner.pubkey(),
            group_seed,
            token_owner,
            mint,
            Some(memo.to_string()),
        )
        .unwrap();

        let mut tx = Transaction::new_with_payer(&[instruction], Some(&state.promo_owner.pubkey()));
        tx.try_partial_sign(&[&state.promo_owner], txd.message.recent_blockhash)
            .unwrap();
        let serialized = bincode::serialize(&tx).unwrap();
        let transaction = base64::encode(serialized);

        assert_eq!(
            parsed_response,
            burn_delegated_promo::ResponseData {
                transaction,
                message: MESSAGE.to_owned(),
            }
        );
    }

    // #[tokio::test]
    // async fn test_get_burn_delegated_promo_tx() {
    //     dotenv::dotenv().ok();

    //     // ok to be devnet, only pulling blockhash - will succeed even if localnet validator not running
    //     let state = State::new(Cluster::Devnet);
    //     let app = create_app(Cluster::Devnet);
    //     let token_owner = Pubkey::new_unique();
    //     let mint = Pubkey::new_unique();

    //     let data = get_mint_promo_tx::Data {
    //         account: token_owner.to_string(),
    //     };

    //     let message = urlencoding::encode(MESSAGE);

    //     let response = app
    //         .oneshot(
    //             Request::builder()
    //                 .method(Method::POST)
    //                 .uri(format!(
    //                     "/promo/burn-delegated/{}/{}",
    //                     mint.to_string(),
    //                     message.into_owned(),
    //                 ))
    //                 .header(header::CONTENT_TYPE, "application/json")
    //                 .body(Body::from(serde_json::to_vec(&data).unwrap()))
    //                 .unwrap(),
    //         )
    //         .await
    //         .unwrap();

    //     assert_eq!(response.status(), StatusCode::OK);

    //     let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    //     let parsed_response: burn_delegated_promo::ResponseData =
    //         serde_json::from_slice(&body).unwrap();

    //     let txd: Transaction = bincode::deserialize(
    //         &base64::decode::<String>(parsed_response.transaction.clone()).unwrap(),
    //     )
    //     .unwrap();

    //     let instruction = create_burn_delegated_promo_instruction(
    //         state.promo_owner.pubkey(),
    //         token_owner,
    //         mint,
    //         state.platform,
    //         None,
    //     )
    //     .unwrap();

    //     let mut tx = Transaction::new_with_payer(&[instruction], Some(&state.promo_owner.pubkey()));
    //     tx.try_partial_sign(&[&state.promo_owner], txd.message.recent_blockhash)
    //         .unwrap();
    //     let serialized = bincode::serialize(&tx).unwrap();
    //     let transaction = base64::encode(serialized);

    //     assert_eq!(
    //         parsed_response,
    //         burn_delegated_promo::ResponseData {
    //             transaction,
    //             message: MESSAGE.to_owned(),
    //         }
    //     );
    // }

    #[tokio::test]
    async fn test_create_buyxproduct_promo() {
        dotenv::dotenv().ok();

        // This test requires a local validator to be running. Whereas the other tests return prepared
        // transactions, this one sends a transaction to create a Promo on chain.
        if let Ok(_) = TcpListener::bind("127.0.0.1:8899".parse::<SocketAddr>().unwrap()) {
            assert!(false, "localnet validator not started")
        }

        let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(create_app(Cluster::Localnet).into_make_service())
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
            }
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
            )
            .text(
                "memo",
                serde_json::json!({"reference": "tester", "memo": "have a great day"}).to_string(),
            );

        let client = reqwest::Client::new();
        let response = client
            .post(format!("http://{}/promo/create", addr))
            .multipart(form)
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();

        println!("{:?}", &response);
        assert!(&response
            .as_object()
            .unwrap()
            .get("result")
            .unwrap()
            .as_str()
            .is_some());
    }

    #[tokio::test]
    async fn test_create_buyxcurrency_promo() {
        dotenv::dotenv().ok();

        // This test requires a local validator to be running. Whereas the other tests return prepared
        // transactions, this one sends a transaction to create a Promo on chain.
        if let Ok(_) = TcpListener::bind("127.0.0.1:8899".parse::<SocketAddr>().unwrap()) {
            assert!(false, "localnet validator not started")
        }

        let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(create_app(Cluster::Localnet).into_make_service())
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
            "name": "buyXCurrency",
            "symbol": "CURR",
            "description": "bokoup test promo - currency",
            "attributes": [
                {
                    "trait_type": "promoType",
                    "value": "buyXCurrencyGetYPercent",
                },
                {
                    "trait_type": "buyXCurrency",
                    "value": 200,
                },
                {
                    "trait_type": "getYPercent",
                    "value": 10
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
                "name": "Currency Promo",
                "family": "Test Merchant Promos"
            }
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
            )
            .text(
                "memo",
                serde_json::json!({"reference": "tester", "memo": "have a great day"}).to_string(),
            );

        let client = reqwest::Client::new();
        let response = client
            .post(format!("http://{}/promo/create", addr))
            .multipart(form)
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();

        println!("{:?}", &response);
        assert!(&response
            .as_object()
            .unwrap()
            .get("result")
            .unwrap()
            .as_str()
            .is_some());
    }
}
