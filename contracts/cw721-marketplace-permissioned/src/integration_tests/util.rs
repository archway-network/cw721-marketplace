#![cfg(test)]
use std::hash::Hash;
use std::collections::HashSet;
use serde::{de::DeserializeOwned, Serialize};

use cosmwasm_std::{
    Addr, BalanceResponse as BalanceResponseBank, BankQuery, Coin, Empty, from_binary, Querier, QueryRequest, 
    StdError, to_binary, Uint128, WasmQuery,
};
use cw_multi_test::{
    App, Contract, ContractWrapper, Executor,
};

use cw20::Cw20Coin;
use cw721_base::{
    msg::InstantiateMsg as Cw721InstantiateMsg
};

use crate::msg::InstantiateMsg;

static DENOM: &str = "aarch";

pub fn mock_app() -> App {
    App::default()
}

pub fn contract_swap721() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub fn contract_cw20() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

pub fn contract_cw721() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw721_base::entry::execute,
        cw721_base::entry::instantiate,
        cw721_base::entry::query,
    );
    Box::new(contract)
}

pub fn create_swap(router: &mut App, owner: &Addr, cw721: Addr) -> Addr {
    
    let swap_id = router.store_code(contract_swap721());
    let msg = InstantiateMsg {
        admin: owner.clone(),
        denom: DENOM.into(),
        cw721: vec![cw721],
    };
    let swap_addr = router
        .instantiate_contract(swap_id, owner.clone(), &msg, &[], "swap721",None)
        .unwrap();
    swap_addr
}

pub fn create_cw721(router: &mut App,minter: &Addr) -> Addr {
    let cw721_id = router.store_code(contract_cw721());
    let msg = Cw721InstantiateMsg {
        name: "TESTNFT".to_string(),
        symbol: "TSNFT".to_string(),
        minter: String::from(minter),
    };   
    let contract = router
        .instantiate_contract(cw721_id, minter.clone(), &msg, &[], "swap721",None)
        .unwrap();    
    contract
}

pub fn mint_native(app: &mut App, beneficiary: String, amount: Uint128) {
    app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: beneficiary,
            amount: vec![Coin {
                denom: DENOM.to_string(),
                amount: amount,
            }],
        },
    ))
    .unwrap();
}

pub fn create_cw20(
    router: &mut App,
    owner: &Addr,
    name: String,
    symbol: String,
    balance: Uint128,
) -> Addr {
    // set up cw20 contract with some tokens
    let cw20_id = router.store_code(contract_cw20());
    let msg = cw20_base::msg::InstantiateMsg {
        name: name,
        symbol: symbol,
        decimals: 2,
        initial_balances: vec![Cw20Coin{
            address: owner.to_string(),
            amount: balance,
        }],
        mint: None,
        marketing:None
    };
    let addr = router
        .instantiate_contract(cw20_id, owner.clone(), &msg, &[], "CASH",None)
        .unwrap();
    addr
}

pub fn query<M,T>(router: &mut App, target_contract: Addr, msg: M) -> Result<T, StdError>
    where
        M: Serialize + DeserializeOwned,
        T: Serialize + DeserializeOwned,
    {
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: target_contract.to_string(),
            msg: to_binary(&msg).unwrap(),
        }))
    }

pub fn bank_query(app: &App, address: &Addr) -> Coin {
    let req: QueryRequest<BankQuery> = QueryRequest::Bank(BankQuery::Balance { 
        address: address.to_string(), 
        denom: DENOM.to_string() 
    });
    let res = app.raw_query(&to_binary(&req).unwrap()).unwrap().unwrap();
    let balance: BalanceResponseBank = from_binary(&res).unwrap();
    return balance.amount;
}

pub fn has_unique_elements<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}