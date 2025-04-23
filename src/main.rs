use clap::Parser;
use clewdr::{
    self, BANNER,
    config::{CONFIG_NAME, ClewdrConfig},
    cookie_manager::CookieManager,
    error::ClewdrError,
    state::ClientState,
    utils::{CLEWDR_DIR, LOG_DIR},
};
use colored::Colorize;
use tracing::warn;
use tracing_subscriber::{
    Registry,
    fmt::{self, time::ChronoLocal},
    layer::SubscriberExt,
};

/// Async main function using tokio runtime
#[tokio::main]
async fn main() -> Result<(), ClewdrError> {
    let _ = enable_ansi_support::enable_ansi_support();
    // parse command line arguments
    clewdr::Args::parse();
    // setup dir
    let _ = CLEWDR_DIR;
    // set up logging time format
    let timer = ChronoLocal::new("%H:%M:%S%.3f".to_string());
    // set up logging
    let file_appender = tracing_appender::rolling::daily(LOG_DIR, "clewdr.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = Registry::default()
        .with(
            fmt::Layer::default()
                .with_writer(file_writer)
                .with_timer(timer.clone()),
        )
        .with(
            fmt::Layer::default()
                .with_writer(std::io::stdout)
                .with_timer(timer),
        );

    tracing::subscriber::set_global_default(subscriber).expect("unable to set global subscriber");

    println!("{}", *BANNER);
    // load config from file
    let config = ClewdrConfig::load()?;

    let updater = clewdr::update::ClewdrUpdater::new(config.clone())?;
    if let Err(e) = updater.check_for_updates().await {
        warn!("Update check failed: {}", e);
    }

    // print the address
    let addr = format!("http://{}", config.address());
    let api_addr = format!("{}/v1", addr);
    println!(
        "Config dir: {}",
        CLEWDR_DIR.join(CONFIG_NAME).display().to_string().blue()
    );
    println!("API address: {}", api_addr.green());
    println!("Web address: {}", addr.green());
    println!("{}", config);

    // initialize the application state
    let tx = CookieManager::start(config.clone());
    let state = ClientState::new(config, tx);
    // build axum router
    // create a TCP listener
    let addr = state.config.address().to_string();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let router = clewdr::router::RouterBuilder::new(state).build();
    // serve the application
    axum::serve(listener, router).await?;
    Ok(())
}
