#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    Binary, Deps, DepsMut, entry_point, Env, MessageInfo, Reply, Response, StdResult, 
    SubMsgResult, to_binary,
};

use crate::execute::{
    execute_create, execute_cancel, execute_finish, execute_update, 
    execute_add_cw721, execute_remove_cw721, execute_update_config, execute_withdraw_fees
};
use crate::query::{
    query_config, query_details, query_list, query_swap_total, query_swaps, query_swaps_by_creator,
    query_swaps_by_denom, query_swaps_by_payment_type, query_swaps_by_price, query_swaps_of_token,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG, SwapType};
use crate::error::ContractError;

use cw2::{get_contract_version, set_contract_version};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:archid-marketplace";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Max fee percentage 30%
    let fee_percentage: u64 = if msg.fee_percentage > 30 { 0_u64 } else { msg.fee_percentage };

    let config = Config {
        admin: msg.admin,
        denom: msg.denom.clone(),
        cw721: msg.cw721,
        fees: fee_percentage,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("denom", msg.denom)
        .add_attribute("fees", msg.fee_percentage.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // Swap entry points
        ExecuteMsg::Create(msg) => execute_create(deps, env, info, msg),
        ExecuteMsg::Finish(msg) => execute_finish(deps, env, info, msg),
        ExecuteMsg::Update(msg) => execute_update(deps, env, info, msg),
        ExecuteMsg::Cancel(msg) => execute_cancel(deps, env, info, msg),
        
        // Admin only entry points
        ExecuteMsg::UpdateConfig { config } => execute_update_config(deps, env, info, config),
        ExecuteMsg::AddNft(msg) => execute_add_cw721(deps, env, info, msg),
        ExecuteMsg::RemoveNft(msg) => execute_remove_cw721(deps, env, info, msg),
        ExecuteMsg::Withdraw(msg) => execute_withdraw_fees(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::List { start_after, limit } => {
            to_binary(&query_list(deps, start_after, limit)?)
        },
        QueryMsg::Details { id } => {
            to_binary(&query_details(deps, id)?)
        },
        QueryMsg::GetTotal { swap_type } => {
            to_binary(&query_swap_total(deps, swap_type)?)
        },
        QueryMsg::GetOffers { page, limit } => {
            to_binary(&query_swaps(deps, SwapType::Offer, page, limit)?)
        },
        QueryMsg::GetListings { page, limit } => {
            to_binary(&query_swaps(deps, SwapType::Sale, page, limit)?)
        }
        QueryMsg::ListingsOfToken { token_id, cw721, swap_type, page, limit } => {
            to_binary(&query_swaps_of_token(deps, token_id, cw721, swap_type, page, limit)?)
        }
        QueryMsg::SwapsOf { address, swap_type, page, limit } => {
            to_binary(&query_swaps_by_creator(deps, address, swap_type, page, limit)?)
        }
        QueryMsg::SwapsByPrice { min, max, swap_type, page, limit } => {
            to_binary(&query_swaps_by_price(deps, min, max, swap_type, page, limit)?)
        }
        QueryMsg::SwapsByDenom { payment_token, swap_type, page, limit } => {
            to_binary(&query_swaps_by_denom(deps, payment_token, swap_type, page, limit)?)
        }
        QueryMsg::SwapsByPaymentType { cw20, swap_type, page, limit } => {
            to_binary(&query_swaps_by_payment_type(deps, cw20, swap_type, page, limit)?)
        }
        QueryMsg::Config {} => {
            to_binary(&query_config(deps)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.result {
        SubMsgResult::Ok(_) => Ok(Response::default()),
        SubMsgResult::Err(_) => Err(ContractError::Unauthorized {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let original_version = get_contract_version(deps.storage)?;
    let name = CONTRACT_NAME.to_string();
    let version = CONTRACT_VERSION.to_string();
    if original_version.contract != name {
        return Err(ContractError::InvalidInput {});
    }
    if original_version.version >= version {
        return Err(ContractError::InvalidInput {});
    }
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::Addr;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR
    };

    // Instantiation works
    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();

        // Instantiate an empty contract
        let instantiate_msg = InstantiateMsg {
            admin: Addr::unchecked(MOCK_CONTRACT_ADDR),
            denom: "aarch".into(),
            cw721: vec![Addr::unchecked(MOCK_CONTRACT_ADDR)],
            fee_percentage: 0_u64,
        };
        let info = mock_info("anyone", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}