# CW721 Single Collection Marketplace
Permits the listing and offering on an NFT collection.

# Index
<details>
<summary>Expand</summary>

<!-- TOC -->
* [CW721 Single Collection Marketplace](#cw721-single-collection-marketplace)
* [Index](#index)
  * [Instantiation](#instantiation)
  * [Messages](#messages)
    * [Create](#create)
    * [Finish](#finish)
    * [Cancel](#cancel)
    * [Update](#update)
    * [UpdateConfig - Permissioned](#updateconfig---permissioned)
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
  * [PageResult](#pageresult)
  * [CW721Swap](#cw721swap)
  * [Expiration](#expiration)
    * [AtHeight](#atheight)
    * [AtTime](#attime)
    * [Never](#never)
  * [SwapType](#swaptype)
<!-- TOC -->
</details>

## Instantiation

| Name           | Type            | Description                               |
|----------------|-----------------|-------------------------------------------|
| admin          | String(Address) | Address allowed to do privileged messages |
| denom          | String          | Token denom for native token listings     |
| cw721          | String(Address) | NFT Collection allowed in the marketplace |
| fee_percentage | u64             | Percentage fee cut, ie: 1 = 1%            |

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

| Name           | Type            | Description                               |
|----------------|-----------------|-------------------------------------------|
| admin          | String(Address) | Address allowed to do privileged messages |
| denom          | String          | Token denom for native token listings     |
| cw721          | String(Address) | NFT Collection allowed in the marketplace |
| fee_percentage | u64             | Percentage fee cut, ie: 1 = 1%            |

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

<details>
<summary>Result</summary>

| Name  | Type         | Description      |
|-------|--------------|------------------|
| swaps | String array | List of swap IDs |

</details>

---
### GetTotal
Count total listings, supports counting a specific  type of listing, returns a number

| Name      | Type                           | Description      |
|-----------|--------------------------------|------------------|
| swap_type | Optional [SwapType](#SwapType) | Swap type filter |

---
### GetOffers

| Name  | Type            | Description            |
|-------|-----------------|------------------------|
| page  | Optional number | Pagination             |
| limit | Optional number | Limit how many results |

Returns a list of [PageResult](#PageResult)

---
### GetListings
| Name  | Type            | Description            |
|-------|-----------------|------------------------|
| page  | Optional number | Pagination             |
| limit | Optional number | Limit how many results |

Returns a list of [PageResult](#PageResult)

---
### ListingsOfToken

| Name      | Type                           | Description            |
|-----------|--------------------------------|------------------------|
| token_id  | String                         | NFT ID                 |
| swap_type | Optional [SwapType](#SwapType) | Swap type filter       |
| page      | Optional number                | Pagination             |
| limit     | Optional number                | Limit how many results |


Returns a list of [PageResult](#PageResult)

---
### SwapsOf

| Name      | Type                           | Description                         |
|-----------|--------------------------------|-------------------------------------|
| address   | String(Address)                | Swaps created by a specific address |
| swap_type | Optional [SwapType](#SwapType) | Swap type filter                    |
| page      | Optional number                | Pagination                          |
| limit     | Optional number                | Limit how many results              |

Returns a list of [PageResult](#PageResult)

---
### SwapsByPrice

| Name      | Type                           | Description             |
|-----------|--------------------------------|-------------------------|
| min       | Optional String(number)        | Minimum price to return |
| max       | Optional String(number)        | Maximum price to return |
| swap_type | Optional [SwapType](#SwapType) | Swap type filter        |
| page      | Optional number                | Pagination              |
| limit     | Optional number                | Limit how many results  |

Returns a list of [PageResult](#PageResult)

---
### SwapsByDenom

| Name          | Type                           | Description            |
|---------------|--------------------------------|------------------------|
| payment_token | Optional String(Address)       | Filter by CW20 token   |
| swap_type     | Optional [SwapType](#SwapType) | Swap type filter       |
| page          | Optional number                | Pagination             |
| limit         | Optional number                | Limit how many results |

Returns a list of [PageResult](#PageResult)

---
### SwapsByPaymentType

| Name      | Type                           | Description            |
|-----------|--------------------------------|------------------------|
| cw20      | bool                           | Filter payment type    |
| swap_type | Optional [SwapType](#SwapType) | Swap type filter       |
| page      | Optional number                | Pagination             |
| limit     | Optional number                | Limit how many results |

Returns a list of [PageResult](#PageResult)

---
### Details
Return the details of the specified listing

| Name | Type   | Description |
|------|--------|-------------|
| id   | String | Listing ID  |

<details>
<summary>Result</summary>

| Name          | Type                      | Description                 |
|---------------|---------------------------|-----------------------------|
| creator       | String(Address)           | Listing creator             |
| payment_token | Optional String(Address)  | Cw20 token if applicable    |
| token_id      | String                    | NFT ID                      |
| expires       | [Expiration](#Expiration) | Listing expiration date     |
| price         | String(Number)            | Amount offered or requested |
| swap_types    | [SwapType](#SwapType)     | Listing type                |


</details>

---
### Config
Query the contract's config, returns:

| Name           | Type            | Description                               |
|----------------|-----------------|-------------------------------------------|
| admin          | String(Address) | Address allowed to do privileged messages |
| denom          | String          | Token denom for native token listings     |
| cw721          | String(Address) | NFT Collection allowed in the marketplace |
| fee_percentage | u64             | Percentage fee cut, ie: 1 = 1%            |

---

## PageResult

| Name  | Type                             | Description           |
|-------|----------------------------------|-----------------------|
| swaps | Array of [CW721Swap](#CW721Swap) | Queried items         |
| page  | number                           | Current page          |
| total | number                           | Total values returned |


---

## CW721Swap

| Name          | Type                      | Description                 |
|---------------|---------------------------|-----------------------------|
| id            | String                    | Listing ID                  |
| creator       | String(Address)           | Creator address             |
| contract      | String(Address)           | NFT collection              |
| payment_token | Optional String(Address)  | CW20 contract               |
| token_id      | String                    | NFT ID                      |
| expires       | [Expiration](#Expiration) | Listing expiration date     |
| price         | String(Number)            | Requested or offered amount |
| swap_type     | [SwapType](#SwapType)     | Listing type                |


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
