# Aura Chain

Blockchain implementation for Aura OS featuring hybrid consensus and identity-first architecture.

## 🏗️ Architecture
- **Consensus**: PoA (system calls) + PoS (user transactions)  
- **Identity**: DID-based accounts with social recovery
- **Privacy**: ZK-proofs for system operations
- **Storage**: Hybrid on-chain/off-chain model

## 🚀 Quick Start

```bash
# Clone and build
git clone https://github.com/aura-os-org/aura-chain
cd aura-chain

# Build blockchain node
cargo build --release

# Run local testnet
./target/release/aura-chain --dev --tmp
