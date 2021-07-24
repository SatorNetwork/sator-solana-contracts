
# Overview

Viewer staking contract


## Dev
```
cargo  build-bpf; cargo fmt --all;cargo clippy --all;cargo test-bpf; cargo doc --open --no-deps
```


## Spec

https://nftprojectsboosty.atlassian.net/browse/SAT-225


```
The stacking contract should lock the Sator tokens (for now - test token) of the user for a special period of time. For the test purpose, the stacking period could be - 30 min- 1 h- 2 hIn the prod version, the timing lock should be taken from the tokenomic doc of the Sator.Depends on how much the user’s stakes, and the period, he gets a special rank and multiplier coefficient, that will be used in getting rewards.Stacking tiers:- Genin - if user stake 100 tokens for any period of time- Chunin - if user stake 200 tokens for at least 1h- Jonin - if user stake 300 tokens for at least 2h- Ninja - if user stake 500 tokens for at least 2hThere should be a possibility to add more tokens to the already steked. The increased amount of the tokens could change the tier of the user.Acceptance criteria:1. Contract developed and deployed to the Solana chain.2. Contract works with test token - 13kBuVtxUT7CeddDgHfe61x3YdpBWTCKeB2Zg2LC4dab
```

https://docs.google.com/document/d/1UnyGdzy--txmfoL5sragqE2LBa6dVHcp/edit



