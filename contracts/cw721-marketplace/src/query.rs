use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Deps, Order, StdResult, Uint128};
use cw_storage_plus::Bound;

use crate::msg::{DetailsResponse, ListResponse};
use crate::state::{all_swap_ids, Config, CONFIG, CW721Swap, SWAPS, SwapType};
use crate::utils::{calculate_page_params, PageParams};

// Pagination query result format for filtered swap queries
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PageResult {
    pub swaps: Vec<CW721Swap>,
    pub page: u32,
    pub total: u128,
}

// Default and Max page sizes for paginated queries
const MAX_LIMIT: u32 = 100;
const DEFAULT_LIMIT: u32 = 10;

pub fn query_details(deps: Deps, id: String) -> StdResult<DetailsResponse> {
    let swap = SWAPS.load(deps.storage, &id)?;
    let details = DetailsResponse {
        creator: swap.creator,
        contract: swap.nft_contract,
        payment_token: swap.payment_token,
        token_id: swap.token_id,
        expires: swap.expires,
        price: swap.price,
        swap_type: swap.swap_type,
    };
    Ok(details)
}

pub fn query_list(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.as_ref().map(|s| Bound::exclusive(s.as_str()));

    Ok(ListResponse {
        swaps: all_swap_ids(deps.storage, start, limit)?,
    })
}

pub fn query_swap_total(deps: Deps, side: Option<SwapType>) -> StdResult<u128> {
    let swaps: Result<Vec<(String, CW721Swap)>, cosmwasm_std::StdError> = SWAPS
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let results: Vec<CW721Swap> = if let Some(swap_type) = side {
        swaps
            .unwrap()
            .into_iter()
            .map(|t| t.1)
            .filter(|item| { item.swap_type == swap_type })
            .collect()
    } else {
        swaps
            .unwrap()
            .into_iter()
            .map(|t| t.1)
            .collect()
    };
    
    Ok(results.len() as u128)
}

pub fn query_swaps(
    deps: Deps,
    side: SwapType, 
    page: Option<u32>, 
    limit: Option<u32>,
) -> StdResult<PageResult> {
    let swaps: Result<Vec<(String, CW721Swap)>, cosmwasm_std::StdError> = SWAPS
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let results: Vec<CW721Swap> = swaps
        .unwrap()
        .into_iter()
        .map(|t| t.1)
        .filter(|item| { item.swap_type == side })
        .collect();

    let paging: PageParams = calculate_page_params(page, limit, results.len() as u32)?;
    let res = PageResult {
        swaps: results[paging.start..paging.end].to_vec(),
        page: paging.page,
        total: paging.total,
    };

    Ok(res)
}

pub fn query_swaps_of_token(
    deps: Deps,
    token_id: String,
    cw721: Addr,
    side: Option<SwapType>, 
    page: Option<u32>, 
    limit: Option<u32>,
) -> StdResult<PageResult> {
    let swaps: Result<Vec<(String, CW721Swap)>, cosmwasm_std::StdError> = SWAPS
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let results: Vec<CW721Swap> = if let Some(swap_type) = side {
        swaps
            .unwrap()
            .into_iter()
            .map(|t| t.1)
            .filter(|item| {
                item.token_id == token_id
                && item.nft_contract == cw721
                && item.swap_type == swap_type
            })
            .collect()
    } else {
        swaps
            .unwrap()
            .into_iter()
            .map(|t| t.1)
            .filter(|item| { 
                item.token_id == token_id 
                && item.nft_contract == cw721
            })
            .collect()
    };

    let paging: PageParams = calculate_page_params(page, limit, results.len() as u32)?;
    let res = PageResult {
        swaps: results[paging.start..paging.end].to_vec(),
        page: paging.page,
        total: paging.total,
    };

    Ok(res)
}

pub fn query_swaps_by_creator(
    deps: Deps, 
    address: Addr,
    swap_type: Option<SwapType>,
    cw721: Option<Addr>,
    page: Option<u32>,
    limit: Option<u32>,
) -> StdResult<PageResult> {
    let side: SwapType = swap_type.unwrap_or(SwapType::Sale);
    let swaps: Result<Vec<(String, CW721Swap)>, cosmwasm_std::StdError> = SWAPS
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let results: Vec<CW721Swap> = if let Some(contract) = cw721 {
        swaps
            .unwrap()
            .into_iter()
            .map(|t| t.1)
            .filter(|item| {
                item.creator == address
                && item.swap_type == side
                && item.nft_contract == contract
            })
            .collect()
    } else {
        swaps
            .unwrap()
            .into_iter()
            .map(|t| t.1)
            .filter(|item| {
                item.creator == address
                && item.swap_type == side
            })
            .collect()
    };

    let paging: PageParams = calculate_page_params(page, limit, results.len() as u32)?;
    let res = PageResult {
        swaps: results[paging.start..paging.end].to_vec(),
        page: paging.page,
        total: paging.total,
    };

    Ok(res)
}

pub fn query_swaps_by_price(
    deps: Deps, 
    min: Option<Uint128>, 
    max: Option<Uint128>, 
    swap_type: Option<SwapType>,
    cw721: Option<Addr>,
    page: Option<u32>,
    limit: Option<u32>,
) -> StdResult<PageResult> {
    let min: Uint128 = min.unwrap_or(Uint128::from(0_u32));
    let side: SwapType = swap_type.unwrap_or(SwapType::Sale);
    let swaps: Result<Vec<(String, CW721Swap)>, cosmwasm_std::StdError> = SWAPS
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    // With Max range filter
    let mut results: Vec<CW721Swap> = if let Some(max_value) = max {
        swaps
            .unwrap()
            .into_iter()
            .map(|t| t.1)
            .filter(|item| {
                item.price.u128() >= min.u128()
                && item.price.u128() <= max_value.u128()
                && item.swap_type == side
            })
            .collect()
    } else {
        swaps
            .unwrap()
            .into_iter()
            .map(|t| t.1)
            .filter(|item| {
                item.price.u128() >= min.u128()
                && item.swap_type == side
            })
            .collect()
    };

    // If limited to a collection scope
    results = if let Some(contract) = cw721 {
        results
            .into_iter()
            .filter(|item| {
                item.nft_contract == contract  
            })
            .collect()
    } else {
        results
    };

    let paging: PageParams = calculate_page_params(page, limit, results.len() as u32)?;
    let res = PageResult {
        swaps: results[paging.start..paging.end].to_vec(),
        page: paging.page,
        total: paging.total,
    };

    Ok(res)
}

pub fn query_swaps_by_denom(
    deps: Deps, 
    payment_token: Option<Addr>, 
    swap_type: Option<SwapType>,
    cw721: Option<Addr>,
    page: Option<u32>,
    limit: Option<u32>,
) -> StdResult<PageResult> {
    let side: SwapType = swap_type.unwrap_or(SwapType::Sale);
    let swaps: Result<Vec<(String, CW721Swap)>, cosmwasm_std::StdError> = SWAPS
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    // Requested cw20 denom
    let mut results: Vec<CW721Swap> = if let Some(token_addr) = payment_token {
        swaps
            .unwrap()
            .into_iter()
            .map(|t| t.1)
            .filter(|item| {
                item.payment_token.clone().unwrap() == token_addr
                && item.swap_type == side
            })
            .collect()
    // Native ARCH denom
    } else {
        swaps
            .unwrap()
            .into_iter()
            .map(|t| t.1)
            .filter(|item| {
                item.payment_token.is_none()
                && item.swap_type == side
            })
            .collect()
    };

    // If limited to a collection scope
    results = if let Some(contract) = cw721 {
        results
            .into_iter()
            .filter(|item| {
                item.nft_contract == contract  
            })
            .collect()
    } else {
        results
    };

    let paging: PageParams = calculate_page_params(page, limit, results.len() as u32)?;
    let res = PageResult {
        swaps: results[paging.start..paging.end].to_vec(),
        page: paging.page,
        total: paging.total,
    };

    Ok(res)
}

pub fn query_swaps_by_payment_type(
    deps: Deps, 
    cw20: bool,
    swap_type: Option<SwapType>,
    cw721: Option<Addr>,
    page: Option<u32>,
    limit: Option<u32>,
) -> StdResult<PageResult> {
    let side: SwapType = swap_type.unwrap_or(SwapType::Sale);
    let swaps: Result<Vec<(String, CW721Swap)>, cosmwasm_std::StdError> = SWAPS
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    // cw20 swap
    let mut results: Vec<CW721Swap> = if cw20 {
        swaps
            .unwrap()
            .into_iter()
            .map(|t| t.1)
            .filter(|item| {
                item.payment_token.is_some()
                && item.swap_type == side
            })
            .collect()
    // ARCH swap
    } else {
        swaps
            .unwrap()
            .into_iter()
            .map(|t| t.1)
            .filter(|item| {
                item.payment_token.is_none()
                && item.swap_type == side
            })
            .collect()
    };

    // If limited to a collection scope
    results = if let Some(contract) = cw721 {
        results
            .into_iter()
            .filter(|item| {
                item.nft_contract == contract  
            })
            .collect()
    } else {
        results
    };

    let paging: PageParams = calculate_page_params(page, limit, results.len() as u32)?;
    let res = PageResult {
        swaps: results[paging.start..paging.end].to_vec(),
        page: paging.page,
        total: paging.total,
    };

    Ok(res)
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(config)
}