use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    BlockInfo, Addr, Order, Uint128, Storage, StdResult,
};
use cw_storage_plus::{
    Bound, Item, Map,
};

use cw20::{Expiration};
use utils::prelude::CW721Swap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub denom: String,
    pub cw721: Addr,
    pub fees: u64,
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


pub const SWAPS: Map<&str, CW721Swap> = Map::new("cw721_swap");
pub const CONFIG: Item<Config> = Item::new("config");