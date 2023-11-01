use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    BlockInfo, Addr, Order, Uint128, Storage, StdResult,
};
use cw_storage_plus::{
    Bound, Item, Map,
};

use cw20::{Expiration};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub denom: String,
    pub cw721: Vec<Addr>,
    pub fees: u64,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum SwapType {
    Offer,
    Sale
}
// swap type of false equals offer, swap type of true equals buy
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CW721Swap {
    pub creator: Addr,
    pub nft_contract: Addr,
    pub payment_token: Option<Addr>,
    pub token_id: String,
    pub expires: Expiration,
    pub price: Uint128,
    pub swap_type: SwapType,
}

pub fn all_swap_ids<'a>(
    storage: &dyn Storage,
    start: Option<Bound<'a, &'a str>>,
    limit: usize,
) -> StdResult<Vec<String>> {
    SWAPS
        .keys(storage, start, None, Order::Ascending)
        .take(limit)
        .collect()
}

impl CW721Swap {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires.is_expired(block)
    }
}

pub const SWAPS: Map<&str, CW721Swap> = Map::new("cw721_swap");
pub const CONFIG: Item<Config> = Item::new("config");