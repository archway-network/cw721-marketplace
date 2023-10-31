use cosmwasm_std::{Coin, DepsMut, Env, MessageInfo, Order, Response};

use crate::state::{CW721Swap, Config, CONFIG, SWAPS, SwapType};
use crate::utils::{
    check_sent_required_payment, query_name_owner, handle_swap_transfers,
};
use crate::msg::{CancelMsg, SwapMsg, UpdateMsg, UpdateNftMsg};
use crate::error::ContractError;

pub fn execute_create(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: SwapMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // If expired, revert
    if msg.expires.is_expired(&env.block) {
        return Err(ContractError::Expired {});
    }
    // If no cw721 permission, revert
    if !config.cw721.contains(&msg.cw721) {
        return Err(ContractError::Unauthorized {});
    }

    let has_payment_token = msg.payment_token.is_some();
    // SwapType::Sale
    if msg.swap_type == SwapType::Sale {
        let owner = query_name_owner(&msg.token_id, &msg.cw721, &deps).unwrap();
        if owner.owner != info.sender {
            return Err(ContractError::Unauthorized {});
        }
    // SwapType::Offer
    } else if msg.swap_type == SwapType::Offer && !has_payment_token {
        return Err(ContractError::InvalidPaymentToken {});
    }
    let swap = CW721Swap {
        creator: info.sender,
        nft_contract: msg.cw721,
        payment_token: msg.payment_token,
        token_id: msg.token_id,
        expires: msg.expires,
        price: msg.price,
        swap_type: msg.swap_type,
    };

    // Try to store it, fail if the id already exists (unmodifiable swaps)
    SWAPS.update(deps.storage, &msg.id, |existing| match existing {
        None => Ok(swap.clone()),
        Some(_) => Err(ContractError::AlreadyExists {}),
    })?;

    let payment_token: String = if has_payment_token {
        swap.payment_token.unwrap().to_string()
    } else {
        config.denom
    };

    Ok(Response::new()
        .add_attribute("action", "create")
        .add_attribute("token_id", swap.token_id)
        .add_attribute("payment_token", payment_token)
        .add_attribute("price", swap.price))
}

pub fn execute_update(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateMsg,
) -> Result<Response, ContractError> {
    let swap = SWAPS.load(deps.storage, &msg.id)?;

    // Only creator can update swap
    if info.sender != swap.creator {
        return Err(ContractError::Unauthorized {});
    }

    // For security reasons, creator, nft_contract, token_id,  
    // payment_token and swap_type should not be updatable
    // E.g. only price and expiration can be modified
    let swap = CW721Swap {
        creator: swap.creator,
        nft_contract: swap.nft_contract,
        payment_token: swap.payment_token,
        token_id: swap.token_id,
        expires: msg.expires,
        price: msg.price,
        swap_type: swap.swap_type,
    };
    // Remove legacy swap and save updated swap
    SWAPS.remove(deps.storage, &msg.id);
    SWAPS.save(deps.storage, &msg.id, &swap)?;

    Ok(Response::new()
        .add_attribute("action", "update")
        .add_attribute("swap_id", &msg.id)
        .add_attribute("token_id", &swap.token_id))
}

pub fn execute_finish(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: SwapMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let swap = SWAPS.load(deps.storage, &msg.id)?;
    // If expired, revert
    if swap.expires.is_expired(&env.block) {
        return Err(ContractError::Expired {});
    }
    // If no cw721 permission, revert
    if !config.cw721.contains(&msg.cw721) {
        return Err(ContractError::Unauthorized {});
    }

    // If swapping for native `aarch`
    // check payment conditions satisfied
    if swap.payment_token.is_none() {
        let required_payment = Coin {
            denom: config.denom.clone(),
            amount: swap.price,
        };
        check_sent_required_payment(&info.funds, Some(required_payment))?;

        // Native aarch offers not allowed
        if swap.swap_type == SwapType::Offer {
            return Err(ContractError::InvalidInput {});
        }
    }
  
    let transfer_results = match msg.swap_type {
        SwapType::Offer => handle_swap_transfers(&info.sender, &swap.creator, swap.clone(), &info.funds, config.denom.clone())?,
        SwapType::Sale => handle_swap_transfers(&swap.creator, &info.sender, swap.clone(), &info.funds, config.denom.clone())?,
    };

    // Remove all swaps for this token_id 
    // (as they're no longer invalid)
    let swaps: Result<Vec<(String, CW721Swap)>, cosmwasm_std::StdError> = SWAPS
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    for swap in swaps.unwrap().iter() {
        if swap.1.token_id == msg.token_id {
            SWAPS.remove(deps.storage, &swap.0);
        }
    }
    
    let payment_token: String = if msg.payment_token.is_some() {
        msg.payment_token.unwrap().to_string()
    } else {
        config.denom
    };

    Ok(Response::new()
        .add_attribute("action", "finish")
        .add_attribute("token_id", msg.token_id)
        .add_attribute("payment_token", payment_token)
        .add_attribute("price", msg.price)
        .add_messages(transfer_results))
}

pub fn execute_cancel(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: CancelMsg,
) -> Result<Response, ContractError> {
    let swap = SWAPS.load(deps.storage, &msg.id)?;
    if info.sender != swap.creator {
        return Err(ContractError::Unauthorized {});
    }

    SWAPS.remove(deps.storage, &msg.id);

    Ok(Response::new()
        .add_attribute("action", "cancel")
        .add_attribute("swap_id", msg.id))
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    config_update: Config,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.save(deps.storage, &config_update)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn execute_add_cw721(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateNftMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    if config.cw721.contains(&msg.cw721) {
        return Err(ContractError::InvalidInput {});
    }

    config.cw721.push(msg.cw721.clone());

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "add_cw721")
        .add_attribute("cw721", msg.cw721))
}

pub fn execute_remove_cw721(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateNftMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    if !config.cw721.contains(&msg.cw721) {
        return Err(ContractError::InvalidInput {});
    }

    config.cw721.retain(|contract| *contract != msg.cw721.clone());

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "add_cw721")
        .add_attribute("cw721", msg.cw721))
}