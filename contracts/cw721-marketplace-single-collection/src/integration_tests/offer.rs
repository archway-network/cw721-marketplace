#![cfg(test)]
use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::Executor;

use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, Expiration};
use cw721::OwnerOfResponse;
use cw721_base::{
    msg::ExecuteMsg as Cw721ExecuteMsg, msg::QueryMsg as Cw721QueryMsg, Extension, MintMsg,
};

use crate::integration_tests::util::{create_cw20, create_cw721, create_swap, mock_app, query};
use crate::msg::{ExecuteMsg, FinishSwapMsg, SwapMsg};
use crate::state::SwapType;

// cw721_owner accepts an offer for some cw20 from cw20_owner
// XXX: cw20 spending approvals will only work for one swap at a time
// unless the dapp does some logic to calculate the approval cumulatively
// for all swaps of the cw20 token in question. Unclear how to manage this
// with expiration date and with Offers being consumed by the NFT owner.
#[test]
fn test_cw20_offer_accepted() {
    let mut app = mock_app();

    // Swap owner deploys
    let swap_admin = Addr::unchecked("swap_deployer");
    // cw721_owner owns the cw721
    let cw721_owner = Addr::unchecked("original_owner");
    // cw20_owner owns the cw20
    let cw20_owner = Addr::unchecked("cw20_owner");

    // cw721_owner creates the cw721
    let nft = create_cw721(&mut app, &cw721_owner);

    // swap_admin creates the swap contract
    let swap = create_swap(&mut app, &swap_admin, nft.clone());
    let swap_inst = swap.clone();

    // cw20_owner creates a cw20 coin
    let cw20 = create_cw20(
        &mut app,
        &cw20_owner,
        "testcw".to_string(),
        "tscw".to_string(),
        Uint128::from(100000_u32),
    );
    let cw20_inst = cw20.clone();

    // cw721_owner mints a cw721
    let token_id = "petrify".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();
    let mint_msg = Cw721ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: cw721_owner.to_string(),
        token_uri: Some(token_uri.clone()),
        extension: None,
    });
    let _res = app
        .execute_contract(cw721_owner.clone(), nft.clone(), &mint_msg, &[])
        .unwrap();

    // Bidding buyer (cw20_owner) must approve swap contract to spend their cw20
    let cw20_approve_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: swap.to_string(),
        amount: Uint128::from(100000_u32),
        expires: None,
    };
    let _res = app
        .execute_contract(
            cw20_owner.clone(),
            cw20_inst.clone(),
            &cw20_approve_msg,
            &[],
        )
        .unwrap();

    // Bidding buyer (cw20_owner) creates an offer
    let creation_msg = SwapMsg {
        id: "firstswap".to_string(),
        payment_token: Some(Addr::unchecked(cw20)),
        token_id: token_id.clone(),
        expires: Expiration::from(cw20::Expiration::AtHeight(384798573487439743)),
        price: Uint128::from(100000_u32),
        swap_type: SwapType::Offer,
    };
    let finish_msg = FinishSwapMsg {
        id: creation_msg.id.clone(),
    };

    let _res = app
        .execute_contract(
            cw20_owner.clone(),
            swap_inst.clone(),
            &ExecuteMsg::Create(creation_msg),
            &[],
        )
        .unwrap();

    // cw721_owner must approve the swap contract to spend their NFT
    let nft_approve_msg = Cw721ExecuteMsg::Approve::<Extension> {
        spender: swap.to_string(),
        token_id: token_id.clone(),
        expires: None,
    };
    let _res = app
        .execute_contract(cw721_owner.clone(), nft.clone(), &nft_approve_msg, &[])
        .unwrap();

    // cw721_owner accepts the cw20 buyer's offer
    let _res = app
        .execute_contract(
            cw721_owner.clone(),
            swap_inst.clone(),
            &ExecuteMsg::Finish(finish_msg),
            &[],
        )
        .unwrap();

    // cw20_owner has received the NFT
    let owner_query: OwnerOfResponse = query(
        &mut app,
        nft.clone(),
        Cw721QueryMsg::OwnerOf {
            token_id: token_id,
            include_expired: None,
        },
    )
    .unwrap();
    assert_eq!(owner_query.owner, cw20_owner);

    // cw721_owner has received the cw20 amount
    let balance_query: BalanceResponse = query(
        &mut app,
        cw20_inst,
        Cw20QueryMsg::Balance {
            address: cw721_owner.to_string(),
        },
    )
    .unwrap();
    assert_eq!(balance_query.balance, Uint128::from(100000_u32));
}

#[test]
fn test_cw20_offer_exploit() {
    let mut app = mock_app();

    // Swap owner deploys
    let swap_admin = Addr::unchecked("swap_deployer");
    // cw721_owner owns the cw721
    let cw721_owner = Addr::unchecked("original_owner");
    // cw20_owner owns the cw20
    let cw20_owner = Addr::unchecked("cw20_owner");
    // Exploit addr
    let bad_actor = Addr::unchecked("bad_actor");

    // cw721_owner creates the cw721
    let nft = create_cw721(&mut app, &cw721_owner);

    // swap_admin creates the swap contract
    let swap = create_swap(&mut app, &swap_admin, nft.clone());
    let swap_inst = swap.clone();

    // cw20_owner creates a cw20 coin
    let cw20 = create_cw20(
        &mut app,
        &cw20_owner,
        "testcw".to_string(),
        "tscw".to_string(),
        Uint128::from(100000_u32),
    );
    let cw20_inst = cw20.clone();

    // cw721_owner mints a cw721
    let token_id = "petrify".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();
    let mint_msg = Cw721ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: cw721_owner.to_string(),
        token_uri: Some(token_uri.clone()),
        extension: None,
    });
    let _res = app
        .execute_contract(cw721_owner.clone(), nft.clone(), &mint_msg, &[])
        .unwrap();

    // Bidding buyer (cw20_owner) must approve swap contract to spend their cw20
    let cw20_approve_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: swap.to_string(),
        amount: Uint128::from(100000_u32),
        expires: None,
    };
    let _res = app
        .execute_contract(
            cw20_owner.clone(),
            cw20_inst.clone(),
            &cw20_approve_msg,
            &[],
        )
        .unwrap();

    // Bidding buyer (cw20_owner) creates an offer
    let creation_msg = SwapMsg {
        id: "firstswap".to_string(),
        payment_token: Some(Addr::unchecked(cw20)),
        token_id: token_id.clone(),
        expires: Expiration::from(cw20::Expiration::AtHeight(384798573487439743)),
        price: Uint128::from(100000_u32),
        swap_type: SwapType::Offer,
    };
    let finish_msg = FinishSwapMsg {
        id: creation_msg.id.clone(),
    };

    let _res = app
        .execute_contract(
            cw20_owner.clone(),
            swap_inst.clone(),
            &ExecuteMsg::Create(creation_msg),
            &[],
        )
        .unwrap();

    // cw721_owner must approve the swap contract to spend their NFT
    let nft_approve_msg = Cw721ExecuteMsg::Approve::<Extension> {
        spender: swap.to_string(),
        token_id: token_id.clone(),
        expires: None,
    };
    let _res = app
        .execute_contract(cw721_owner.clone(), nft.clone(), &nft_approve_msg, &[])
        .unwrap();

    // cw721_owner accepts the cw20 buyer's offer
    let res = app.execute_contract(
        bad_actor.clone(),
        swap_inst.clone(),
        &ExecuteMsg::Finish(finish_msg),
        &[],
    );

    assert!(res.is_err())
}
