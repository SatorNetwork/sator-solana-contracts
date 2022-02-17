
## Deploy

### BEFORE

- Upgrade Solana CLI tools to match solana cluster version
- Make sure deployer account has 2 SOL for new program

### MAINNET

```
solana cluster-version --url https://api.mainnet-beta.solana.com

solana program deploy ./target/deploy/sator_stake_viewer.so --url https://api.mainnet-beta.solana.com  --program-id ./.solana/StXzdE4rS9UYF8H6NMgXFdKmTYuvDd2zNfqCzo3xWry.json --keypair ./.solana/yab38PLcDqYqyYZS6b16nz9JGSVBQmaPsB2EN7s8JJy.json

solana program close --buffers --url https://api.mainnet-beta.solana.com  --keypair ./.solana/yab38PLcDqYqyYZS6b16nz9JGSVBQmaPsB2EN7s8JJy.json
```
### DEVNET

```
solana program deploy ./target/deploy/sator_stake_viewer.so --url https://api.devnet.solana.com  --program-id ./.solana/StXzdE4rS9UYF8H6NMgXFdKmTYuvDd2zNfqCzo3xWry.json --keypair ./.solana/yab38PLcDqYqyYZS6b16nz9JGSVBQmaPsB2EN7s8JJy.json
```