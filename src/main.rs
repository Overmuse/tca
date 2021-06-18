use anyhow::Result;
use tca::{run, Settings};
use tracing::subscriber::set_global_default;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv::dotenv();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    set_global_default(subscriber).expect("Failed to set subscriber");
    let settings = Settings::new()?;
    run(settings).await
}
