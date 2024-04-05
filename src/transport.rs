use std::time::Duration;

// https://github.com/paradigmxyz/reth/blob/crates/net/eth-wire/src/p2pstream.rs#L9
use futures::{
    SinkExt,
    StreamExt,
};
use reth_ecies::stream::ECIESStream;
use reth_ecies::ECIESError;
use reth_eth_wire::{
    CanDisconnect,
    DisconnectReason,
};
use reth_primitives::{
    BytesMut,
    B512,
};
use secp256k1::SecretKey;
use tokio::net::TcpStream;
use tracing::{
    error,
    info,
};

use crate::cfg::Node;

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("node timeout after {after}ms")]
    NodeTimeout {
        node: Node,
        after: u64,
    },
    #[error("ecies error: {0}")]
    EciesError(#[from] ECIESError),
    #[error("no response from {node:?}")]
    NoResponse {
        node: Node,
    },
}

/// Transport layer for establishing stream connections to nodes.
pub struct Transport {
    stream: ECIESStream<TcpStream>,
}

impl Transport {
    /// Connect to a node with a timeout.
    pub async fn connect(
        node: &Node,
        key: SecretKey,
        timeout: u64,
    ) -> Result<Self, TransportError> {
        info!("connecting to {node:?}");

        match tokio::time::timeout(
            Duration::from_millis(timeout),
            TcpStream::connect(format!("{}:{}", node.ip, node.port)),
        )
        .await
        {
            Ok(stream) => {
                match stream {
                    Ok(tcp_stream) => {
                        info!("connected to {node:?}");
                        let remote_peer_id: B512 = node.id.parse().unwrap();

                        // https://github.com/paradigmxyz/reth/blob/examples/manual-p2p/src/main.rs#L87
                        let stream = ECIESStream::connect(tcp_stream, key, remote_peer_id).await?;

                        Ok(Self {
                            stream,
                        })
                    }
                    Err(e) => Err(TransportError::IOError(e).into()),
                }
            }
            Err(_) => {
                Err(TransportError::NodeTimeout {
                    node: node.clone(),
                    after: timeout,
                })
            }
        }
    }

    /// Send a message to a node with a timeout.
    /// Optionally ignore the response so that we don't raise an error if we don't get one.
    pub async fn send(
        &mut self,
        node: Node,
        message: BytesMut,
        timeout: u64,
        ignore_response: bool,
    ) -> Result<BytesMut, TransportError> {
        info!("sending message to {node:?}");

        // https://github.com/paradigmxyz/reth/blob/crates/net/eth-wire/src/p2pstream.rs#L100
        self.stream.send(message.into()).await?;

        if ignore_response {
            info!("no response expected from {node:?}");
            return Ok(BytesMut::new());
        }

        match tokio::time::timeout(Duration::from_millis(timeout), self.stream.next())
            .await
            .map_err(|_| {
                TransportError::NodeTimeout {
                    node: node.clone(),
                    after: timeout,
                }
            })? {
            Some(b) => b.map_err(|b| b.into()),
            None => {
                error!("no response from {node:?}");
                Err(TransportError::NoResponse {
                    node,
                })
            }
        }
    }

    /// Disconnect stream, closing sink.
    pub async fn disconnect(&mut self) {
        self.stream.disconnect(DisconnectReason::ClientQuitting).await.unwrap();
    }
}
