# Overview

This document describes how to make Solana assets more secured. How not to loose your tokens and SOLs.

There are to ways to loose, loose your wallet private information or share it with malicious others.

## Schemas 

There are several ways to make signatures secure.

### Cold wallet

Use cold device (not your day to day driver) just to create Solana wallet. 

Run full encryption on that device. Android, Windows, Mac - all can encrypt.

Use open source wallets which cannot restore your keys anyhow.

Prefer console wallet.

When your need some tokens or SOL for operations, transfer them to Hot wallet available on your work device.

Encrypt and export `Cold Wallet`, share storage with people your trust. Make sure that at leas on of them tried to use wallet on cold device too.

Consider to use CLI tools which allows at least N of M people your shared encrypted wallet to decrypt to get access. 

### Wrapped wSOL and multisignature tokens

Use CLI tools to wrap SOL into token. 

Transfer wrapped SOLs (which are tokens) or token to accounts accesible under by N of M signatures of people of trust. 

To transfer from Cold Wallet serveral people, like 2 of 4, should sign transfer.

Multi signature wallets can use `Cold wallets` too. 

### Are hardware wallets safe? 

Generally - no.

If hardware is broken you loose access. 

If hardware provider can produce new key than he has access to your wallet. Or how could he manage to restore it?

If hardware producer is compromised, than you are too. 

Harwware wallets may be good with `Wrapped wSOL and multisignature tokens`.

Probably your hardware producer is not open source and does not revela if it uses N of M signatures or governance process to reproduce your hadware wallet.

## New Cold Wallet

If you thin wallet was compromised, create new Cold Wallet. 

Hackers may wait long before use keys. 

Never share screen while operating on private keys visuall. If that heppend, create new Cold Wallet.

## Contracts

Check list if you share money with contracts:

- deployer company must have good history of safety and community collaboration
- program is sealed and immutable
- program is open source with high score of being mature (mature team, long commit history, many tests, etc)
- program is audited by third party
- you personally deployed program or person you trust
- program uses multisignature deploy if mutable
- you tested program on small amounts
- program has reproducible builds, you downloaded program from solana, built it locally, and hashes of both match 

