# CW721 Permissioned Marketplace
Allows for the listing and offering on NFTs. This particular contract supports multiple NFT collections in a 
permissioned manner, meaning that the admin decides which collections are allowed on the marketplace.

# Index
<!-- TOC -->
* [CW721 Permissioned Marketplace](#cw721-permissioned-marketplace)
* [Index](#index)
  * [Instantiation](#instantiation)
  * [Messages](#messages)
    * [Create](#create)
    * [Finish](#finish)
    * [Cancel](#cancel)
    * [Update](#update)
    * [UpdateConfig - Permissioned](#updateconfig---permissioned)
    * [AddNft - Permissioned](#addnft---permissioned)
    * [RemoveNft - Permissioned](#removenft---permissioned)
    * [Withdraw - Permissioned](#withdraw---permissioned)
  * [Queries](#queries)
    * [List](#list)
    * [GetTotal](#gettotal)
    * [GetOffers](#getoffers)
    * [GetListings](#getlistings)
    * [ListingsOfToken](#listingsoftoken)
    * [SwapsOf](#swapsof)
    * [SwapsByPrice](#swapsbyprice)
    * [SwapsByDenom](#swapsbydenom)
    * [SwapsByPaymentType](#swapsbypaymenttype)
    * [Details](#details)
    * [Config](#config)
  * [Expiration](#expiration)
    * [AtHeight](#atheight)
    * [AtTime](#attime)
    * [Never](#never)
  * [SwapType](#swaptype)
<!-- TOC -->

## Instantiation

| Name           | Type                  | Description                                |
|----------------|-----------------------|--------------------------------------------|
| admin          | String(Address)       | Address allowed to do privileged messages  |
| denom          | String                | Token denom for native token listings      |
| cw721          | String(Address) Array | NFT Collections allowed in the marketplace |
| fee_percentage | u64                   | Percentage fee cut, ie: 1 = 1%             |

## Messages

### Create
Create an offer or a sale for a specific NFT. 
Each type of listing has specific caveats that must be followed to meet the original design's user experience.

* Offer
  * Can only use CW20 token
  * User must give this contract an allowance of equal or greater number than the offered amount
* Sale
  * User must give this contract transfer permissions

| Name          | Type                      | Description                                                                    |
|---------------|---------------------------|--------------------------------------------------------------------------------|
| id            | String                    | Created ID for the listing, cannot be a currently existing ID                  |
| cw721         | String(Address)           | NFT contract, must be supported by the marketplace                             |
| payment_token | String(Address)           | Optional cs20 address, defaults to aarch if empty                              |
| token_id      | String                    | Nft token id                                                                   |
| expires       | [Expiration](#Expiration) | When the listing will expire                                                   |
| price         | String(Uint128)           | When a sale its the requested amount, when its an offer its the offered amount |
| swap_type     | [SwapType](#SwapType)     | The type of listing                                                            |

---

### Finish
Finalize the listing. Permission to do this varies depending the listing type.

* Sale
  * Buyer must trigger this
  * Must give this contract an allowance of equal or greater number than the offered amount
* Offer
  * Seller must trigger this
  * Must give this contract transfer permissions

| Name | Type   | Description       |
|------|--------|-------------------|
| id   | String | Sale / Offer swap |

---

### Cancel
Cancels the listing, can only be triggered by listing creator.

| Name | Type   | Description       |
|------|--------|-------------------|
| id   | String | Sale / Offer swap |

---

### Update
Update the listing, can only be triggered by listing creator.

| Name    | Type                      | Description                                                                    |
|---------|---------------------------|--------------------------------------------------------------------------------|
| id      | String                    | Sale / Offer swap                                                              |
| expires | [Expiration](#Expiration) | When the listing will expire                                                   |
| price   | String(Uint128)           | When a sale its the requested amount, when its an offer its the offered amount |

---

### UpdateConfig - Permissioned
Updates the contract config set at instantiation.

| Name           | Type                  | Description                                |
|----------------|-----------------------|--------------------------------------------|
| admin          | String(Address)       | Address allowed to do privileged messages  |
| denom          | String                | Token denom for native token listings      |
| cw721          | String(Address) Array | NFT Collections allowed in the marketplace |
| fee_percentage | u64                   | Percentage fee cut, ie: 1 = 1%             |

---

### AddNft - Permissioned
Add an allowed NFT contract to be offered in the marketplace

| Name  | Type            | Description           |
|-------|-----------------|-----------------------|
| cw721 | String(Address) | NFT collection to add |

---

### RemoveNft - Permissioned
Remove an allowed NFT contract from the marketplace

| Name  | Type            | Description                |
|-------|-----------------|----------------------------|
| cw721 | String(Address) | NFT collection to withdraw |

---

### Withdraw - Permissioned
Withdraw tokens earned by the contract through sale fees

| Name          | Type                     | Description                            |
|---------------|--------------------------|----------------------------------------|
| amount        | String(Number)           | Amount to withdraw                     |
| denom         | String                   | Native coin denom                      |
| payment_token | Optional String(Address) | Optional cw20 address to withdraw from |

---

## Queries

### List
Get all pending swaps

| Name        | Type            | Description                    |
|-------------|-----------------|--------------------------------|
| start_after | Optional String | Limit which ID to start after  |
| limit       | Optional number | Limit how many swaps to return |

#### Returns

| Name  | Type         | Description      |
|-------|--------------|------------------|
| swaps | String array | List of swap IDs |


---
### GetTotal
#### Returns

---
### GetOffers
#### Returns

---
### GetListings
#### Returns

---
### ListingsOfToken
#### Returns

---
### SwapsOf
#### Returns

---
### SwapsByPrice
#### Returns

---
### SwapsByDenom
#### Returns

---
### SwapsByPaymentType
#### Returns

---
### Details
#### Returns

---
### Config
#### Returns

---

## Expiration
When something can expire, the contents can be one of three.

### AtHeight
Will expire when given height is greater or equal than the current block height
```json
{
    "at_height": 10
}
```

### AtTime
Will expire when given time is greater or equal than the current block height
```json
{
  "at_time": "epoch number"
}
```

### Never
Will never expire
```json
"never"
```

---

## SwapType
Represents the type of transaction going through, can be one of `Offer` which is an offer to someone's NFT and `Sale`
which is a listing to sell an owned NFT
