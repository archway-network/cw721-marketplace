# Cw721 Marketplaces

In this repository you'll find contracts for easily deploying the following types of marketplaces

- [Single collection](/contracts/cw721-marketplace-single-collection)
    - Allows swapping NFTs from a specific cw721 contract for native tokens or cw20s
    - Only contract admin can set or update the cw721 contract address used as default in swaps
- [Permissioned](/contracts/cw721-marketplace-permissioned)
    - Allows swapping NFTs from a curated list of cw721 contracts for native tokens or cw20s
    - Only contract admin can set or update the list of cw721 contract addresses
- [Multiple collections (open)](/contracts/cw721-marketplace)
    - Allows swapping any cw721 NFT for native tokens or cw20s

