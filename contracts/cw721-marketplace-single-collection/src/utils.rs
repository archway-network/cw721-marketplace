use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{
    Addr, BalanceResponse, BankMsg, BankQuery, Coin, CosmosMsg, Decimal, DepsMut, Env, from_json, 
    QueryRequest, to_json_binary, StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};

use cw20::Cw20ExecuteMsg;
use cw721_base::{QueryMsg as Cw721QueryMsg};
use cw721::OwnerOfResponse;
use cw721_base::{msg::ExecuteMsg as Cw721ExecuteMsg, Extension};
use utils::prelude::CW721Swap;

use crate::state::{CONFIG};
use crate::error::ContractError;

// Default and Max page sizes for paginated queries
const MAX_LIMIT: u32 = 100;
const DEFAULT_LIMIT: u32 = 10;

// Pagination query pagaination parameters for filtered swap queries
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PageParams {
    pub start: usize,
    pub end: usize,
    pub page: u32,
    pub total: u128,
}

// Fee split result
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FeeSplit {
    pub marketplace: Uint128,
    pub seller: Uint128,
}

// Read utils
pub fn query_name_owner(
    id: &str,
    cw721: &Addr,
    deps: &DepsMut,
) -> Result<OwnerOfResponse, StdError> {
    let query_msg = Cw721QueryMsg::OwnerOf {
        token_id: id.to_owned(),
        include_expired: None,
    };
    let req = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: cw721.to_string(),
        msg: to_json_binary(&query_msg).unwrap(),
    });
    let res: OwnerOfResponse = deps.querier.query(&req)?;
    Ok(res)
}

pub fn calculate_page_params(
    page: Option<u32>,
    limit: Option<u32>,
    total_results: u32,
) -> Result<PageParams, StdError> {
    let page: u32 = page.unwrap_or(0_u32);
    let mut limit: u32 = limit.unwrap_or(DEFAULT_LIMIT);
    // Calculate dynamic limit and last page size
    if total_results < limit {
        limit = total_results;
    } else if limit < DEFAULT_LIMIT {
        limit = DEFAULT_LIMIT;
    } else if limit > MAX_LIMIT {
        limit = MAX_LIMIT;
    }
    let modulo = if total_results > 0 { total_results % limit } else { 0 };
    let last_page = if total_results == 0 {
        0 
    } else if modulo > 0 { 
        total_results / limit 
    } else {
        total_results / limit - 1 
    };
    let page_size: u32 = if page == last_page { 
        match modulo {
            0 => limit,
            _ => modulo,
        }
    } else { 
        limit 
    };

    // Results
    let start = (page * limit) as usize;
    let end = (start as u32 + page_size) as usize;
    let res = PageParams {
        start,
        end, 
        page,
        total: total_results as u128,
    };

    Ok(res)
}

pub fn check_sent_required_payment(
    sent: &[Coin],
    required: Option<Coin>,
) -> Result<(), ContractError> {
    if let Some(required_coin) = required {
        let required_amount = required_coin.amount.u128();
        if required_amount > 0 {
            let sent_sufficient_funds = sent.iter().any(|coin| {
                // check if a given sent coin matches denom
                // and has sufficient amount
                coin.denom == required_coin.denom && coin.amount.u128() >= required_amount
            });

            if sent_sufficient_funds {
                return Ok(());
            } else {
                return Err(ContractError::Unauthorized {});
            }
        }
    }
    Ok(())
}

pub fn check_sent_required_payment_exact(
    sent: &[Coin],
    required: Option<Coin>,
) -> Result<(), ContractError> {
    if let Some(required_coin) = required {
        let required_amount = required_coin.amount.u128();
        if required_amount > 0 {
            let sent_exact_funds = sent.iter().any(|coin| {
                // check if a given sent coin matches denom
                // and has sent exactly the required amount
                coin.denom == required_coin.denom && coin.amount.u128() == required_amount
            });

            if sent_exact_funds {
                return Ok(());
            } else {
                return Err(ContractError::ExactFunds {});
            }
        }
    }
    Ok(())
}

pub fn check_contract_balance_ok(
    env: Env,
    deps: &DepsMut,
    required: Coin,
) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let required_denom: String = config.denom;
    if required.denom != required_denom {
        return Err(ContractError::InsufficientBalance {});
    }
    let swap_instance: &Addr = &env.contract.address;
    let required_amount = required.amount.u128();

    // Balance query
    let req: QueryRequest<BankQuery> = QueryRequest::Bank(BankQuery::Balance { 
        address: swap_instance.to_string(),
        denom: required_denom,
    });
    let res = deps.querier.raw_query(&to_json_binary(&req).unwrap()).unwrap().unwrap();
    let query: BalanceResponse = from_json(res).unwrap();
    let balance: Coin = query.amount;
    if balance.amount.u128() < required_amount {
        return Err(ContractError::InsufficientBalance {});
    }

    Ok(())
}

// Write utils
pub fn handle_swap_transfers(
    env: Env,
    nft_sender: &Addr,
    nft_receiver: &Addr,
    details: CW721Swap,
    denom: String,
    fee_split: FeeSplit,
) -> StdResult<Vec<CosmosMsg>> {
    // cw20 swap
    let payment_callback: CosmosMsg = if details.payment_token.is_some() {
        let token_transfer_msg = Cw20ExecuteMsg::TransferFrom {
            owner: nft_receiver.to_string(),
            recipient: nft_sender.to_string(),
            amount: fee_split.seller,
        };

        let cw20_callback: CosmosMsg = WasmMsg::Execute {
            contract_addr: details.payment_token.clone().unwrap().into(),
            msg: to_json_binary(&token_transfer_msg)?,
            funds: vec![],
        }
        .into();
        cw20_callback
    // aarch swap
    } else {
        let payment_funds = ([Coin {
            denom,
            amount: fee_split.seller,
        }])
        .to_vec();
        let aarch_transfer_msg = BankMsg::Send {
            to_address: nft_sender.to_string(),
            amount: payment_funds,
        };

        let aarch_callback: CosmosMsg = cosmwasm_std::CosmosMsg::Bank(aarch_transfer_msg);
        aarch_callback
    };

    let market_callback: Option<CosmosMsg> = if details.payment_token.is_some() && fee_split.marketplace.u128() > 0 { 
        let token_transfer_msg = Cw20ExecuteMsg::TransferFrom {
            owner: nft_receiver.to_string(),
            recipient: env.contract.address.to_string(),
            amount: fee_split.marketplace,
        };

        let cw20_callback: CosmosMsg = WasmMsg::Execute {
            contract_addr: details.payment_token.clone().unwrap().into(),
            msg: to_json_binary(&token_transfer_msg)?,
            funds: vec![],
        }
        .into();
        Some(cw20_callback)
    } else { None };

    let nft_transfer_msg = Cw721ExecuteMsg::<Extension>::TransferNft {
        recipient: nft_receiver.to_string(),
        token_id: details.token_id.clone(),
    };

    let cw721_callback: CosmosMsg = WasmMsg::Execute {
        contract_addr: details.nft_contract.to_string(),
        msg: to_json_binary(&nft_transfer_msg)?,
        funds: vec![],
    }
    .into();

    let mut msgs = vec![cw721_callback, payment_callback];
    if let Some(fees) = market_callback { msgs.push(fees); }

    Ok(msgs)
}

pub fn fee_split(
    deps: &DepsMut,
    swap_price: Uint128,
) -> Result<FeeSplit, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let marketplace: Uint128 = fee_percentage(swap_price, config.fees);
    if marketplace.u128() >= swap_price.into() {
        return Err(ContractError::InvalidInput {});
    }
    let seller: Uint128 = Uint128::from(swap_price.u128() - marketplace.u128());
    let result = FeeSplit { 
        marketplace,
        seller,
    };
    Ok(result)
}

pub fn fee_percentage(amount: Uint128, share_percent: u64) -> Uint128 {
    let share = Decimal::percent(share_percent);
    let amount_decimal = Decimal::from_atomics(amount, 0).unwrap();
    let fee = amount_decimal * share;
    fee.to_uint_floor()
}