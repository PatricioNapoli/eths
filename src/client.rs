use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaChaRng;
use reth_eth_wire::capability::SharedCapabilities;
use reth_eth_wire::protocol::Protocol;
use reth_eth_wire::{
    Capability,
    DisconnectReason,
    EthVersion,
    HelloMessage,
    P2PMessage,
    ProtocolVersion,
};
use reth_primitives::alloy_primitives::private::alloy_rlp::{
    Decodable,
    Encodable,
};
use reth_primitives::{
    pk2id,
    BytesMut,
};
use secp256k1::{
    PublicKey,
    Secp256k1,
    SecretKey,
};
use tokio::task::{
    JoinError,
    JoinSet,
};
use tracing::{
    error,
    info,
    warn,
};

use crate::cfg::{
    Config,
    Node,
};
use crate::transport::{
    Transport,
    TransportError,
};

#[derive(Debug, thiserror::Error)]
pub enum HandshakeError {
    #[error("transport error: {0}")]
    TransportError(#[from] TransportError),
    #[error("tokio task join error: {0}")]
    JoinError(#[from] JoinError),
    #[error("received a message too large from peer")]
    MessageTooBig {
        size: usize,
        max: usize,
    },
    #[error("received disconnect from peer")]
    DisconnectReceived(DisconnectReason),
    #[error("failed to decode response")]
    DecodeFailed(#[from] reth_primitives::alloy_primitives::private::alloy_rlp::Error),
    #[error("non hello response")]
    NonHelloMessage,
}

// https://github.com/paradigmxyz/reth/blob/crates/net/eth-wire/src/p2pstream.rs#L32
pub const MAX_PAYLOAD_SIZE: usize = 16 * 1024 * 1024;

#[derive(Clone)]
pub struct Client {
    config: Config,
    private_key: SecretKey,
    public_key: PublicKey,
}

/// A peer node that replied to a handshake with their capabilities.
#[derive(Debug)]
pub struct Peer {
    node: Node,
    capabilities: Vec<Capability>,
}

impl Client {
    pub fn new(config: Config) -> Self {
        // Could be saved between sessions
        // See node identity: https://github.com/ethereum/devp2p/blob/master/rlpx.md#node-identity
        let secp = Secp256k1::new();

        // ChaCha might be faster than the std thread_rng by a small margin
        // See: https://rust-random.github.io/book/guide-rngs.html#cryptographically-secure-pseudo-random-number-generators-csprngs
        let keypair = secp.generate_keypair(&mut ChaChaRng::from_entropy());

        Self {
            config,
            private_key: keypair.0,
            public_key: keypair.1,
        }
    }

    fn get_client_version() -> &'static str {
        "eths/0.1.0"
    }

    fn get_client_protocol_version() -> ProtocolVersion {
        ProtocolVersion::V5
    }

    /// Get our capabilities.
    /// We "support" all versions from 66 to 68.
    fn get_client_capabilities() -> Vec<Protocol> {
        vec![EthVersion::Eth66.into(), EthVersion::Eth67.into(), EthVersion::Eth68.into()]
    }

    /// Attempt a handshake with all nodes in the config.
    /// Returns a list of candidate peers.
    pub async fn handshake_nodes(&self) -> Result<Vec<Peer>, HandshakeError> {
        let mut set = JoinSet::new();

        for node in &self.config.nodes {
            set.spawn(Client::handshake(node.clone(), self.clone()));
        }

        let mut candidate_peers: Vec<Peer> = vec![];

        while let Some(res) = set.join_next().await {
            match res {
                Ok(r) => {
                    match r {
                        Ok(peer) => {
                            let shared_cap = SharedCapabilities::try_new(
                                Self::get_client_capabilities(),
                                peer.capabilities.clone(),
                            );

                            match shared_cap {
                                Ok(cap) => {
                                    info!(
                                        "found shared capabilities: {:?} with {:?}",
                                        cap.eth_version(),
                                        peer.node
                                    );
                                    candidate_peers.push(peer);
                                }
                                Err(_) => {
                                    warn!(
                                        "no shared capabilities with {:?} - they have {:?}",
                                        peer.node, peer.capabilities
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            warn!("handshake failed for a node: {e}");
                        }
                    }
                }
                Err(e) => {
                    error!("node handshake task join failed: {e}");
                }
            }
        }

        Ok(candidate_peers)
    }

    /// Attempt a handshake with a single node.
    /// Builds an RLP-encoded HelloMessage and sends it to the node.
    async fn handshake(node: Node, client: Client) -> Result<Peer, HandshakeError> {
        info!("starting handshake to {node:?}");
        let mut transport =
            Transport::connect(&node, client.private_key, client.config.timeout).await?;

        let peer_id = pk2id(&client.public_key);
        let hs_msg = HelloMessage::builder(peer_id)
            .client_version(Self::get_client_version())
            .protocol_version(Self::get_client_protocol_version())
            .protocols(Self::get_client_capabilities())
            .build()
            .into_message();

        let mut msg_bytes = BytesMut::new();
        hs_msg.encode(&mut msg_bytes);

        let res = transport.send(node.clone(), msg_bytes, client.config.timeout, false).await?;
        Self::handle_handshake_response(transport, client, node, res).await
    }

    /// Handle the response from a handshake.
    /// Disconnects from peer and returns it with capabilities if successful.
    ///
    /// https://github.com/paradigmxyz/reth/blob/main/crates/net/eth-wire/src/p2pstream.rs#L122
    async fn handle_handshake_response(
        mut transport: Transport,
        client: Client,
        node: Node,
        response: BytesMut,
    ) -> Result<Peer, HandshakeError> {
        if response.len() > MAX_PAYLOAD_SIZE {
            return Err(HandshakeError::MessageTooBig {
                size: response.len(),
                max: MAX_PAYLOAD_SIZE,
            });
        }

        let peer_hello = match P2PMessage::decode(&mut &response[..]) {
            Ok(P2PMessage::Hello(hello)) => {
                info!("received hello from {node:?}");

                Ok(hello)
            }
            Ok(P2PMessage::Disconnect(reason)) => Err(HandshakeError::DisconnectReceived(reason)),
            Err(err) => Err(HandshakeError::DecodeFailed(err)),
            Ok(_) => Err(HandshakeError::NonHelloMessage),
        }?;

        info!("hello message: {peer_hello:?}");

        // We send a disconnect since we are only interested in their capabilities for now
        let mut disconnect_msg = BytesMut::new();
        P2PMessage::Disconnect(DisconnectReason::ClientQuitting).encode(&mut disconnect_msg);

        transport.send(node.clone(), disconnect_msg, client.config.timeout, true).await?;
        transport.disconnect().await;

        Ok(Peer {
            node,
            capabilities: peer_hello.capabilities,
        })
    }
}

#[cfg(test)]
mod tests {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::{
        fmt,
        EnvFilter,
    };

    use crate::cfg::Config;
    use crate::client::Client;

    /// A successful enode might be different in the future
    const ENODE_CSV: &str = concat!(
    "5825b736bb359a0d52ab70867d880ab468ec2b29ff2a743d9d3d3d767869b772a6b455b142ff9339abebe348f2559d4f6dd1ba1219e598ce8e5935065211881e,95.216.64.51,30303");

    #[tokio::test]
    async fn node_handshake() {
        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()))
            .init();

        let config = Config::from_str(ENODE_CSV).unwrap();
        let c = Client::new(config);
        match c.handshake_nodes().await {
            Ok(peers) => {
                assert_eq!(peers.is_empty(), false);
            }
            Err(e) => {
                panic!("{e}");
            }
        }
    }
}
