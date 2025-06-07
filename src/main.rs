use dotenvy::dotenv;
use once_cell::sync::Lazy;
use std::env;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use url_shortener::Config;

mod graphql;
mod server;

static TRACING: Lazy<()> = Lazy::new(|| {
    let max_log_level = env::var("MAX_LOG_LEVEL").unwrap_or_default();

    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::from_level(
            match max_log_level.as_str() {
                "DEBUG" => Level::DEBUG,
                _ => Level::INFO,
            },
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .flatten_event(true)
                .with_file(true)
                .with_level(true)
                .with_line_number(true)
                .with_current_span(true)
                .with_target(true),
        )
        .init();
});

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    Lazy::force(&TRACING);

    let config = envy::from_env::<Config>().unwrap_or_default();

    server::start(config).await?.await
}
