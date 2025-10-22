use env_logger::Env;

mod dto;
mod service;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    service::run_server().await?;
    Ok(())
}
