
# Overview

Viewer staking contract


## Dev
```
cargo  build-bpf; cargo fmt --all;cargo clippy --all;cargo test-bpf; cargo doc --open --no-deps
```


## Old Spec


https://docs.google.com/document/d/1UnyGdzy--txmfoL5sragqE2LBa6dVHcp/edit


## Deploy

```
ACCOUNT=$(solana  address)
solana airdrop 10 $ACCOUNT --url https://api.devnet.solana.com
solana program deploy ./target/deploy/sator_stake_viewer.so --url https://api.devnet.solana.com  --program-id ./program/sator_stake_viewer-keypair.json --keypair /home/dz/validator-keypair.json
https://explorer.solana.com/address/CL9tjeJL38C3eWqd6g7iHMnXaJ17tmL2ygkLEHghrj4u?cluster=devnet
```

### Initalize

- change [cli](/cli) parameters and run