use cosmwasm_std::{Addr, Uint128};
use cw20::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::prelude::{CW721Swap, SwapType};

// Pagination query result format for filtered swap queries
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PageResult {
    pub swaps: Vec<CW721Swap>,
    pub page: u32,
    pub total: u128,
}

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