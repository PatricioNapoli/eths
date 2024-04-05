use argh::FromArgs;
use thiserror::Error;
use tracing::{
    info,
    warn,
};
use tracing_subscriber::{
    filter::LevelFilter,
    fmt,
    prelude::*,
    EnvFilter,
};

use crate::cfg::{
    ConfigError,
    DEFAULT_FILENAME,
    DEFAULT_TIMEOUT,
};
use crate::client::HandshakeError;

mod cfg;
mod client;
mod transport;

#[derive(Debug, Error)]
pub enum Error {
    #[error("config error: {0}")]
    ConfigError(#[from] ConfigError),
    #[error("handshake error: {0}")]
    HandshakeError(#[from] HandshakeError),
}

#[derive(FromArgs, Debug)]
/// ETH node handshake args.
pub struct Args {
    /// public nodes filename override
    #[argh(option, default = "default_file()")]
    pub nodefile: String,

    /// node timeout in ms
    #[argh(option, default = "default_timeout()")]
    pub timeout: u64,
}

fn default_file() -> String {
    DEFAULT_FILENAME.to_string()
}

fn default_timeout() -> u64 {
    DEFAULT_TIMEOUT
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Error> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()))
        .init();

    let args: Args = argh::from_env();
    let config = cfg::Config::from_file(&args.nodefile)?.with_timeout(args.timeout);
    let nodes = config.nodes.len();

    info!("running with {config:?}");
    let p2p: client::Client = client::Client::new(config);

    match p2p.handshake_nodes().await {
        Ok(peers) => {
            if peers.is_empty() {
                warn!("no peers with matching capabilities found!")
            }

            for p in &peers {
                info!("handshake successful for peer: {p:?}");
            }

            info!("handshake complete with {} out of {} nodes", peers.len(), nodes);

            Ok(())
        }
        Err(e) => {
            info!("handshake failed: {e}");
            Err(e.into())
        }
    }
}
