use cosmwasm_std::{
    BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Order, Response,
    to_json_binary, WasmMsg,
};

use cw20::Cw20ExecuteMsg;

use cw721_marketplace_utils::{
    FeeSplit, prelude::CW721Swap, prelude::SwapType
};
use crate::utils::{
    check_sent_required_payment, fee_split, handle_swap_transfers, query_name_owner,
};

use crate::state::{Config, CONFIG, SWAPS};
use crate::msg::{
    CancelMsg, FinishSwapMsg, SwapMsg, UpdateMsg, WithdrawMsg
};
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
        id: msg.id.clone(),
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
        .add_attribute("swap_id", msg.id)
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
        id: swap.id,
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
    msg: FinishSwapMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let swap = SWAPS.load(deps.storage, &msg.id)?;
    // If expired, revert
    if swap.expires.is_expired(&env.block) {
        return Err(ContractError::Expired {});
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

    // Calculate fee split
    let split = if swap.payment_token.is_none() { 
        let funds: Vec<Coin> = info.funds.into_iter()
            .filter(|coin| { coin.denom == config.denom })
            .collect();
        
        fee_split(&deps, funds[0].amount).unwrap_or(FeeSplit::only_seller(funds[0].amount))
    } else {
        fee_split(&deps, swap.price).unwrap_or(FeeSplit::only_seller(swap.price))
    };

    // Do swap transfer
    let transfer_results = match swap.swap_type {
        SwapType::Offer => handle_swap_transfers(
            env,
            &info.sender, 
            &swap.creator, 
            swap.clone(), 
            config.denom.clone(),
            split,
        )?,
        SwapType::Sale => handle_swap_transfers(
            env,
            &swap.creator, 
            &info.sender, 
            swap.clone(), 
            config.denom.clone(),
            split,
        )?,
    };

    // Remove all swaps for this token_id 
    // (as they're no longer valid)
    let swap_data = swap.clone();
    let swaps: Result<Vec<(String, CW721Swap)>, cosmwasm_std::StdError> = SWAPS
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    for swap in swaps.unwrap().iter() {
        if swap.1.token_id == swap_data.token_id && swap.1.nft_contract == swap_data.nft_contract {
            SWAPS.remove(deps.storage, &swap.0);
        }
    }
    
    let payment_token: String = if swap.payment_token.is_some() {
        swap.payment_token.unwrap().to_string()
    } else {
        config.denom
    };

    Ok(Response::new()
        .add_attribute("action", "finish")
        .add_attribute("swap_id", swap.id)
        .add_attribute("token_id", swap.token_id)
        .add_attribute("payment_token", payment_token)
        .add_attribute("price", swap.price)
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

pub fn execute_withdraw_fees(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: WithdrawMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let denom = msg.denom;
    let amount = msg.amount;

    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let transfer_result = if msg.payment_token.is_none() {
        let bank_transfer_msg = BankMsg::Send {
            to_address: info.sender.into(),
            amount: ([Coin { 
                denom: denom.clone(), 
                amount 
            }]).to_vec(),
        };

        let bank_transfer: CosmosMsg = cosmwasm_std::CosmosMsg::Bank(bank_transfer_msg);
        bank_transfer
    } else {
        let cw20_transfer_msg = Cw20ExecuteMsg::Transfer {
            recipient: info.sender.into(),
            amount,
        };
        
        let cw20_transfer: CosmosMsg = cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: msg.payment_token.unwrap().into(),
            msg: to_json_binary(&cw20_transfer_msg)?,
            funds: vec![],
        });
        cw20_transfer
    };

    Ok(Response::new()
        .add_attribute("action", "withdraw")
        .add_attribute("amount", amount.to_string())
        .add_attribute("denom", denom)
        .add_message(transfer_result))
}