use crate::state::Config;
use cosmwasm_std::{Addr, Uint128};
use cw20::Expiration;
use cw721_marketplace_utils::prelude::SwapType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub denom: String,
    pub cw721: Vec<Addr>,
    pub fee_percentage: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Swap entry points
    Create(SwapMsg),
    Finish(FinishSwapMsg),
    Cancel(CancelMsg),
    Update(UpdateMsg),

    // Admin entry points
    UpdateConfig { config: Config },
    AddNft(UpdateNftMsg),
    RemoveNft(UpdateNftMsg),
    Withdraw(WithdrawMsg),
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
pub struct FinishSwapMsg {
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateNftMsg {
    pub cw721: Addr,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawMsg {
    pub amount: Uint128,
    pub denom: String,
    pub payment_token: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // TODO: link to actual structs
    /// Get all swaps (enumerable)
    /// Return type: ListResponse
    List {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    // Count total `SwapType::Offer` or `SwapType::Sale`
    GetTotal {
        swap_type: Option<SwapType>,
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
        cw721: Addr,
        swap_type: Option<SwapType>,
        page: Option<u32>,
        limit: Option<u32>,
    },
    /// Show all swaps created by a specific address
    /// Defaults to SwapType::Sale if no `swap_type`
    SwapsOf {
        address: Addr,
        swap_type: Option<SwapType>,
        cw721: Option<Addr>,
        page: Option<u32>,
        limit: Option<u32>,
    },
    /// Show all swaps of a given price range
    SwapsByPrice {
        min: Option<Uint128>,
        max: Option<Uint128>,
        swap_type: Option<SwapType>,
        cw721: Option<Addr>,
        page: Option<u32>,
        limit: Option<u32>,
    },
    /// Show all swaps of a given denom (contract address)
    /// Defaults to ARCH if no contract is sent
    SwapsByDenom {
        payment_token: Option<Addr>,
        swap_type: Option<SwapType>,
        cw721: Option<Addr>,
        page: Option<u32>,
        limit: Option<u32>,
    },
    /// Show all cw20 swaps, or all ARCH swaps
    SwapsByPaymentType {
        cw20: bool,
        swap_type: Option<SwapType>,
        cw721: Option<Addr>,
        page: Option<u32>,
        limit: Option<u32>,
    },

    /// Returns the details of the named swap, error if not created.
    /// Return type: DetailsResponse.
    Details {
        id: String,
    },

    /// Query Config (useful for determining parameters for ExecuteMsg::UpdateConfig)
    Config {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}
