<div align="center">

<a style="margin-right:15px" href="#"><img src="https://forthebadge.com/images/badges/made-with-rust.svg" alt="Made with Rust"/></a>


<a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-brightgreen.svg" alt="License MIT"/></a>
<a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.77-orange.svg" alt="Rust 1.77"/></a>

</div>

# ETHS

(ETH HandShake) A simple Ethereum client for connecting to enodes through the RLPx protocol.  
Relies on code from ParadigmXYZ's RLPx implementation, namely the ECIES encryption and decryption functions, and the RLPx packet encoding and decoding functions.  
Those functions create the necessary secrets for stream transport, along with Ethereum's MAC custom encryption, and the RLP-encoded HelloMessage for the handshake.  
The client is able to connect to an enode, perform the handshake, and check the matching capabilities (currently "supporting" ETH66 to ETH68, as a mock).

# Details

The toolchain forces the nightly build of rust, due to the usage of experimental cargo fmt styling. However, this styling can be removed, and the stable rust version can be used.  
The nodes to connect to are setup through a .csv file with no header, with the format:
```csv
enode_id,ip,port
```

# Usage

This client requires at least Rust 1.76.0 due to an alloy-rpc-types-trace dependency, and optionally the nightly build for the cargo fmt styling.  
Build has been tested on Debian 11 Bullseye and MacOS 14.0 M1 Max.  
To run the client, you can try the --help argument:
```bash
cargo run -- --help

Usage: eths [--nodefile <nodefile>] [--timeout <timeout>]

ETH node handshake args.

Options:
  --nodefile        public nodes filename override
  --timeout         node timeout in ms
  --help            display usage information
```

By default, the node filename is set to `nodes.csv` (root of the project), and the timeout is set to 2500ms.

NOTE: Your cargo config might require git-fetch-with-cli, see: https://doc.rust-lang.org/cargo/reference/config.html#netgit-fetch-with-cli

# Output

Using the defaults:
```bash
cargo run

2024-04-05T20:29:00.282158Z  INFO eths: running with Config { nodes: [Node { id: "00022472a33bf4be92599db8d2a284599141dcbeea0610f88887e631e5531d90c926aeb1ca003dc4d99ecb1e43c3472d4d2006ebb0c38f51d7b7470c91f767b5", ip: "82.66.183.172", port: 30303 }, Node { id: "5825b736bb359a0d52ab70867d880ab468ec2b29ff2a743d9d3d3d767869b772a6b455b142ff9339abebe348f2559d4f6dd1ba1219e598ce8e5935065211881e", ip: "95.216.64.51", port: 30303 }], timeout: 2500 }
2024-04-05T20:29:00.283083Z  INFO eths::client: starting handshake to Node { id: "00022472a33bf4be92599db8d2a284599141dcbeea0610f88887e631e5531d90c926aeb1ca003dc4d99ecb1e43c3472d4d2006ebb0c38f51d7b7470c91f767b5", ip: "82.66.183.172", port: 30303 }
2024-04-05T20:29:00.283099Z  INFO eths::transport: connecting to Node { id: "00022472a33bf4be92599db8d2a284599141dcbeea0610f88887e631e5531d90c926aeb1ca003dc4d99ecb1e43c3472d4d2006ebb0c38f51d7b7470c91f767b5", ip: "82.66.183.172", port: 30303 }
2024-04-05T20:29:00.283147Z  INFO eths::client: starting handshake to Node { id: "5825b736bb359a0d52ab70867d880ab468ec2b29ff2a743d9d3d3d767869b772a6b455b142ff9339abebe348f2559d4f6dd1ba1219e598ce8e5935065211881e", ip: "95.216.64.51", port: 30303 }
2024-04-05T20:29:00.283236Z  INFO eths::transport: connecting to Node { id: "5825b736bb359a0d52ab70867d880ab468ec2b29ff2a743d9d3d3d767869b772a6b455b142ff9339abebe348f2559d4f6dd1ba1219e598ce8e5935065211881e", ip: "95.216.64.51", port: 30303 }
2024-04-05T20:29:00.520506Z  INFO eths::transport: connected to Node { id: "5825b736bb359a0d52ab70867d880ab468ec2b29ff2a743d9d3d3d767869b772a6b455b142ff9339abebe348f2559d4f6dd1ba1219e598ce8e5935065211881e", ip: "95.216.64.51", port: 30303 }
2024-04-05T20:29:00.533050Z  INFO eths::transport: connected to Node { id: "00022472a33bf4be92599db8d2a284599141dcbeea0610f88887e631e5531d90c926aeb1ca003dc4d99ecb1e43c3472d4d2006ebb0c38f51d7b7470c91f767b5", ip: "82.66.183.172", port: 30303 }
2024-04-05T20:29:00.751855Z  INFO eths::transport: sending message to Node { id: "5825b736bb359a0d52ab70867d880ab468ec2b29ff2a743d9d3d3d767869b772a6b455b142ff9339abebe348f2559d4f6dd1ba1219e598ce8e5935065211881e", ip: "95.216.64.51", port: 30303 }
2024-04-05T20:29:00.753044Z  INFO eths::client: received hello from Node { id: "5825b736bb359a0d52ab70867d880ab468ec2b29ff2a743d9d3d3d767869b772a6b455b142ff9339abebe348f2559d4f6dd1ba1219e598ce8e5935065211881e", ip: "95.216.64.51", port: 30303 }
2024-04-05T20:29:00.753072Z  INFO eths::client: hello message: HelloMessage { protocol_version: V5, client_version: "erigon/v2.59.2-a013ec25/linux-amd64/go1.21.5", capabilities: [Capability { name: "eth", version: 68 }], port: 0, id: 0x5825b736bb359a0d52ab70867d880ab468ec2b29ff2a743d9d3d3d767869b772a6b455b142ff9339abebe348f2559d4f6dd1ba1219e598ce8e5935065211881e }
2024-04-05T20:29:00.753109Z  INFO eths::transport: sending message to Node { id: "5825b736bb359a0d52ab70867d880ab468ec2b29ff2a743d9d3d3d767869b772a6b455b142ff9339abebe348f2559d4f6dd1ba1219e598ce8e5935065211881e", ip: "95.216.64.51", port: 30303 }
2024-04-05T20:29:00.753446Z  INFO eths::transport: no response expected from Node { id: "5825b736bb359a0d52ab70867d880ab468ec2b29ff2a743d9d3d3d767869b772a6b455b142ff9339abebe348f2559d4f6dd1ba1219e598ce8e5935065211881e", ip: "95.216.64.51", port: 30303 }
2024-04-05T20:29:00.753762Z  INFO eths::client: found shared capabilities: Ok(Eth68) with Node { id: "5825b736bb359a0d52ab70867d880ab468ec2b29ff2a743d9d3d3d767869b772a6b455b142ff9339abebe348f2559d4f6dd1ba1219e598ce8e5935065211881e", ip: "95.216.64.51", port: 30303 }
2024-04-05T20:29:00.753806Z  WARN eths::client: handshake failed for a node: transport error: ecies error: stream closed due to not being readable
2024-04-05T20:29:00.753830Z  INFO eths: handshake successful for peer: Peer { node: Node { id: "5825b736bb359a0d52ab70867d880ab468ec2b29ff2a743d9d3d3d767869b772a6b455b142ff9339abebe348f2559d4f6dd1ba1219e598ce8e5935065211881e", ip: "95.216.64.51", port: 30303 }, capabilities: [Capability { name: "eth", version: 68 }] }
2024-04-05T20:29:00.753848Z  INFO eths: handshake complete with 1 out of 2 nodes
```

The nodes currently setup in the repo, serve as an example of a single node that refuses to handshake, and another that successfully completes the handshake with an ETH66 capability.  
When a node refuses to reply, the log will show something similar to the following:
```bash
2024-04-05T20:29:00.753806Z  WARN eths::client: handshake failed for a node: transport error: ecies error: stream closed due to not being readable
```

When a node successfully completes the handshake, the log will show the peer that replied and the capabilities it supports:
```bash
2024-04-05T20:46:56.781064Z  INFO eths::client: received hello from Node { id: "5825b736bb359a0d52ab70867d880ab468ec2b29ff2a743d9d3d3d767869b772a6b455b142ff9339abebe348f2559d4f6dd1ba1219e598ce8e5935065211881e", ip: "95.216.64.51", port: 30303 }
2024-04-05T20:46:56.781114Z  INFO eths::client: hello message: HelloMessage { protocol_version: V5, client_version: "erigon/v2.59.2-a013ec25/linux-amd64/go1.21.5", capabilities: [Capability { name: "eth", version: 68 }], port: 0, id: 0x5825b736bb359a0d52ab70867d880ab468ec2b29ff2a743d9d3d3d767869b772a6b455b142ff9339abebe348f2559d4f6dd1ba1219e598ce8e5935065211881e }

...

2024-04-05T20:29:00.753830Z  INFO eths: handshake successful for peer: Peer { node: Node { id: "5825b736bb359a0d52ab70867d880ab468ec2b29ff2a743d9d3d3d767869b772a6b455b142ff9339abebe348f2559d4f6dd1ba1219e598ce8e5935065211881e", ip: "95.216.64.51", port: 30303 }, capabilities: [Capability { name: "eth", version: 68 }] }
```

However, a node can complete the handshake, but not support the expected capabilities, in which case the log will show the following:
```bash
2024-04-05T20:30:58.820437Z  WARN eths::client: no shared capabilities with Node { id: "5825b736bb359a0d52ab70867d880ab468ec2b29ff2a743d9d3d3d767869b772a6b455b142ff9339abebe348f2559d4f6dd1ba1219e598ce8e5935065211881e", ip: "95.216.64.51", port: 30303 } - they have [Capability { name: "eth", version: 68 }]
```

# Tests

There are tests for the config file and the client handshake, you can run all of them with:
```bash
cargo test
```
Be mindful over several runs, as previously successful nodes might do some rate limiting/throttling, and the client test might fail.