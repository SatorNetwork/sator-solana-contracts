# sator-solana-contracts+
Solana on chain programs for staking and rewards.

For documentations see readme's and `doc` html files.


### References

https://explorer.solana.com/address/2HeykdKjzHKGm2LKHw8pDYwjKPiFEoXAz74dirhUgQvq/metadata

2HeykdKjzHKGm2LKHw8pDYwjKPiFEoXAz74dirhUgQvq

https://www.gate.io/trade/SAO_USDT

## Deploy

## Deploy

```
ACCOUNT=$(solana  address)
solana airdrop 10 $ACCOUNT --url https://api.devnet.solana.com
solana program deploy ./target/deploy/sator_stake_viewer.so --url https://api.devnet.solana.com  --program-id ./sator_stake_viewer-keypair.json --keypair /home/dz/validator-keypair.json
https://explorer.solana.com/address/CL9tjeJL38C3eWqd6g7iHMnXaJ17tmL2ygkLEHghrj4u?cluster=devnet
solana program deploy ./target/deploy/sator_reward.so --url https://api.devnet.solana.com  --program-id ./sator_reward-keypair.json --keypair /home/dz/validator-keypair.json
https://explorer.solana.com/address/DajevvE6uo5HtST4EDguRUcbdEMNKNcLWjjNowMRQvZ1?cluster=devnet
```