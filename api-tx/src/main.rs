use anchor_lang::prelude::Pubkey;
use bpl_api_tx::{create_app, parse_string_to_keypair, utils::solana::Cluster};
use bundlr_sdk::{bundlr::get_pub_info, consts::BUNDLR_DEFAULT_URL};
use clap::Parser;
use solana_sdk::signer::Signer;
use std::{net::SocketAddr, str::FromStr};
use tracing_subscriber::prelude::*;
use url::Url;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value = "l", value_parser)]
    cluster: Cluster,
    #[clap(
        long,
        default_value = "2R7GkXvQQS4iHptUvQMhDvRSNXL8tAuuASNvCYgz3GQW",
        value_parser
    )]
    platform: Pubkey,
    #[clap(long, env = "PLATFORM_SIGNER_KEYPAIR")]
    platform_signer: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "bpl_api_tx=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_ansi(false))
        .init();

    let args = Args::parse();

    let pub_info = get_pub_info(&Url::from_str(BUNDLR_DEFAULT_URL).unwrap())
        .await
        .unwrap();

    let platform_signer = parse_string_to_keypair(&args.platform_signer);
    tracing::debug!(platform_signer = platform_signer.pubkey().to_string());

    let data_url: Url = Url::from_str(match args.cluster {
        Cluster::Devnet => "https://data.api.bokoup.dev/v1/graphql/",
        _ => "https://shining-sailfish-15.hasura.app/v1/graphql/",
    })
    .unwrap();

    let app = create_app(
        args.cluster,
        args.platform,
        platform_signer,
        data_url,
        pub_info,
    );
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
