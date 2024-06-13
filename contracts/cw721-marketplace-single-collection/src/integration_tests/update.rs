#![cfg(test)]
use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::Executor;

use cw20::{Cw20ExecuteMsg, Expiration};
use cw721_base::{msg::ExecuteMsg as Cw721ExecuteMsg, Extension, MintMsg};
use cw721_marketplace_utils::prelude::DetailsResponse;

use crate::integration_tests::util::{create_cw20, create_cw721, create_swap, mock_app, query};
use crate::msg::{ExecuteMsg, QueryMsg, SwapMsg, UpdateMsg};
use crate::state::SwapType;

// Updating a swap of type SwapType::Sale
#[test]
fn test_updating_sales() {
    let mut app = mock_app();

    // Swap owner deploys
    let swap_admin = Addr::unchecked("swap_deployer");
    // cw721_owner owns the cw721
    let cw721_owner = Addr::unchecked("original_owner");

    // cw721_owner creates the cw721
    let nft = create_cw721(&mut app, &cw721_owner);

    // swap_admin creates the swap contract
    let swap = create_swap(&mut app, &swap_admin, nft.clone());
    let swap_inst = swap.clone();

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

    // Create a SwapMsg for creating a swap
    let swap_id: String = "firstswap".to_string();
    let creation_msg = SwapMsg {
        id: swap_id.clone(),
        payment_token: None,
        token_id: token_id.clone(),
        expires: Expiration::from(cw20::Expiration::AtHeight(384798573487439743)),
        price: Uint128::from(1000000000000000000_u128), // 1 ARCH as aarch
        swap_type: SwapType::Sale,
    };

    // Seller (cw721_owner) must approve the swap contract to spend their NFT
    let nft_approve_msg = Cw721ExecuteMsg::Approve::<Extension> {
        spender: swap.to_string(),
        token_id: token_id.clone(),
        expires: None,
    };
    app.execute_contract(cw721_owner.clone(), nft.clone(), &nft_approve_msg, &[])
        .unwrap();

    // cw721 seller (cw721_owner) creates a swap
    let _res = app
        .execute_contract(
            cw721_owner.clone(),
            swap_inst.clone(),
            &ExecuteMsg::Create(creation_msg),
            &[],
        )
        .unwrap();

    // Original swap details (price and expiration) are correct
    let swap_details: DetailsResponse = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::Details {
            id: swap_id.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        swap_details.expires,
        Expiration::from(cw20::Expiration::AtHeight(384798573487439743))
    );
    assert_eq!(swap_details.price, Uint128::from(1000000000000000000_u128));

    // cw721 seller (cw721_owner) updates the swap
    let update_msg = UpdateMsg {
        id: swap_id.clone(),
        expires: Expiration::from(cw20::Expiration::AtHeight(400000000000000000)),
        price: Uint128::from(2000000000000000000_u128),
    };
    let _res = app
        .execute_contract(
            cw721_owner.clone(),
            swap_inst.clone(),
            &ExecuteMsg::Update(update_msg),
            &[],
        )
        .unwrap();

    // Swap details (price and expiration) must be updated
    let swap_details: DetailsResponse = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::Details {
            id: swap_id.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        swap_details.expires,
        Expiration::from(cw20::Expiration::AtHeight(400000000000000000))
    );
    assert_eq!(swap_details.price, Uint128::from(2000000000000000000_u128));
}

// Updating a swap of type SwapType::Offer
#[test]
fn test_updating_offers() {
    let mut app = mock_app();

    // Swap owner deploys
    let swap_admin = Addr::unchecked("swap_deployer");
    // cw721_owner owns the cw721
    let cw721_owner = Addr::unchecked("original_owner");
    // cw20_owner owns wARCH (wrapped ARCH)
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
        "wrapped arch".to_string(),
        "wARCH".to_string(),
        Uint128::from(9000000000000000000_u128), // 9 wARCH
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
        amount: Uint128::from(9000000000000000000_u128),
        expires: None,
    };
    let _res = app
        .execute_contract(cw20_owner.clone(), cw20_inst, &cw20_approve_msg, &[])
        .unwrap();

    // Bidding buyer creates an offer
    let swap_id: String = "firstswap".to_string();
    let creation_msg = SwapMsg {
        id: swap_id.clone(),
        payment_token: Some(Addr::unchecked(cw20)),
        token_id: token_id.clone(),
        expires: Expiration::from(cw20::Expiration::AtHeight(384798573487439743)),
        price: Uint128::from(9000000000000000000_u128), // 9 wARCH
        swap_type: SwapType::Offer,
    };

    let _res = app
        .execute_contract(
            cw20_owner.clone(),
            swap_inst.clone(),
            &ExecuteMsg::Create(creation_msg),
            &[],
        )
        .unwrap();

    // Original swap details (price and expiration) are correct
    let swap_details: DetailsResponse = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::Details {
            id: swap_id.clone(),
        },
    )
    .unwrap();

    assert_eq!(
        swap_details.expires,
        Expiration::from(cw20::Expiration::AtHeight(384798573487439743))
    );
    assert_eq!(swap_details.price, Uint128::from(9000000000000000000_u128));

    // Bidder (cw20_owner) updates the swap
    let update_msg = UpdateMsg {
        id: swap_id.clone(),
        expires: Expiration::from(cw20::Expiration::AtHeight(400000000000000000)),
        price: Uint128::from(1000000000000000000_u128),
    };
    let _res = app
        .execute_contract(
            cw20_owner.clone(),
            swap_inst.clone(),
            &ExecuteMsg::Update(update_msg),
            &[],
        )
        .unwrap();

    // Swap details were correctly updated
    let swap_details: DetailsResponse = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::Details {
            id: swap_id.clone(),
        },
    )
    .unwrap();

    assert_eq!(
        swap_details.expires,
        Expiration::from(cw20::Expiration::AtHeight(400000000000000000))
    );
    assert_eq!(swap_details.price, Uint128::from(1000000000000000000_u128));
}
