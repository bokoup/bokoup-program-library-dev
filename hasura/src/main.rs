use bpl_hasura::{apply_migrations, get_client, reset, DatabaseURL};
use clap::{Parser, Subcommand};
use tokio_postgres::Error;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    #[clap(short, long)]
    quiet: bool,
    #[clap(long, arg_enum, default_value_t = DatabaseURL::default(), value_parser)]
    db_url: DatabaseURL,
}

#[derive(Subcommand)]
enum Commands {
    ResetSchema,
    ApplyMigrations,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    let cli = Cli::parse();
    let mut client = get_client(&cli.db_url.url()).await?;

    if !cli.quiet {
        std::env::set_var("RUST_LOG", "bpl_hasura=trace");
        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(EnvFilter::from_default_env())
            .init();
    }

    match &cli.command {
        Commands::ResetSchema => reset(&mut client).await,
        Commands::ApplyMigrations => apply_migrations(&mut client).await,
    }
}
