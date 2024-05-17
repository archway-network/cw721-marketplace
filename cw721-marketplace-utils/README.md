# Cw721-marketplace-utils

Utility functions, types and helpers intended for use in the following packages:
- `cw721-marketplace`
- `cw721-marketplace-permissioned`
- `cw721-marketplace-single-collection`

This package was created to reduce the amount of redundant code inside the above 3 very similar packages.

### Query

`PageResult{swaps, page, total}` - A formatter struct for paginated swaps data

`ListResponse{swaps}` - Response type for entry point `List`

`DetailsResponse{creator, contract, payment_token, token_id, expires, price, swap_type}` - Response type for entry point `Details`

### Swap

`SwapType{Offer, Sale}` - Enum type for `CW721Swap` that distinguishes whether the `cw721` token is for sale by owner, or being bid on buy a potential buyer.

`CW721Swap{id, creator, nft_contract, payment_token, token_id, expires, price, swap_type}` - Struct for creating or finishing a `cw721` marketplace swap using entry point `Create` or entry point `Finish`

### Fees

`FeeSplit{marketplace, seller}` - A formatter struct for split ratios for marketplaces that collect a fee share of swaps when the `Finish` entry point is executed.

`fee_percentage(amount, share_percent)` - Utility function that bifurcates a price value into a `FeeSplit` when given a swap price and a percentage amount. 

***

XXX Note: This package is a work in progress and part of an unfinished journey of deduplicating redundant code in the cw721 marketplace repostories. 