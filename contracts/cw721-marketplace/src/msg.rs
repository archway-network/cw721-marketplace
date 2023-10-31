use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Uint128};
use cw20::Expiration;
use crate::state::{Config,SwapType};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub denom: String,
    pub fee_percentage: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Create(SwapMsg),
    Finish(SwapMsg),
    Cancel(CancelMsg),
    Update(UpdateMsg),
    UpdateConfig { config: Config, },
    Withdraw { amount: Uint128, },
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CancelMsg {
    pub id: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateMsg {
    pub id: String,
    pub expires: Expiration,
    pub price: Uint128,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SwapMsg {
    pub id: String,
    pub cw721: Addr,
    pub payment_token: Option<Addr>, // Optional cw20 address; if `None` create swap for `aarch`
    pub token_id: String,
    pub expires: Expiration,
    pub price: Uint128,
    pub swap_type: SwapType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Get all swaps (enumerable)
    /// Return type: ListResponse
    List {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    // Count total `SwapType::Offer` or `SwapType::Sale`
    GetTotal {
        swap_type: SwapType,
    },
    /// Get all swaps of type `SwapType::Offer`
    GetOffers {
        page: Option<u32>,
        limit: Option<u32>,
    },
    /// Get all swaps of type `SwapType::Sale`
    GetListings {
        page: Option<u32>,
        limit: Option<u32>,
    },
    /// Get all listings for a token of type `Swap::Sale` and `Swap::Offer` 
    /// or both (`None`)
    ListingsOfToken {
        token_id: String,
        swap_type: Option<SwapType>,
        page: Option<u32>,
        limit: Option<u32>,
    },
    /// Show all swaps created by a specific address
    /// Defaults to SwapType::Sale if no `swap_type`
    SwapsOf { 
        address: Addr,
        swap_type: Option<SwapType>,
        page: Option<u32>,
        limit: Option<u32>,
    },
    /// Show all swaps of a given price range
    SwapsByPrice { 
        min: Option<Uint128>,
        max: Option<Uint128>,
        swap_type: Option<SwapType>,
        page: Option<u32>,
        limit: Option<u32>,
    },
    /// Show all swaps of a given denom (contract address)
    /// Defaults to ARCH if no contract is sent
    SwapsByDenom {
        payment_token: Option<Addr>,
        swap_type: Option<SwapType>,
        page: Option<u32>,
        limit: Option<u32>,
    },
    /// Show all cw20 swaps, or all ARCH swaps
    SwapsByPaymentType {
        cw20: bool,
        swap_type: Option<SwapType>,
        page: Option<u32>,
        limit: Option<u32>,
    },

    /// Returns the details of the named swap, error if not created.
    /// Return type: DetailsResponse.
    Details { id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

// List swaps
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListResponse {
    pub swaps: Vec<String>,
}

// Get details about a swap
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DetailsResponse {
    pub creator: Addr,
    pub contract: Addr,
    pub payment_token: Option<Addr>,
    pub token_id: String,    
    pub expires: Expiration,    
    pub price: Uint128,
    pub swap_type: SwapType,
}