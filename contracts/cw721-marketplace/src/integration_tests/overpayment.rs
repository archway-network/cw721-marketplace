#![cfg(test)]
use cosmwasm_std::{
    Addr, Coin, Uint128,
};
use cw_multi_test::Executor;

use cw20::{
    BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, Expiration,
};
use cw721_base::{
    msg::ExecuteMsg as Cw721ExecuteMsg, Extension, MintMsg, msg::QueryMsg as Cw721QueryMsg,
};
use cw721::OwnerOfResponse;

use crate::integration_tests::util::{
    bank_query, create_cw20, create_cw721, create_swap, mint_native, mock_app, query,
};
use crate::msg::{
    ExecuteMsg, SwapMsg,
};
use crate::state::{SwapType};

static DENOM: &str = "aarch";

// cw721 buyer (arch_owner) overpays
// seller (cw721_owner) receives the full overpaid amount
#[test]
fn test_overpayment_native() {
    let mut app = mock_app();
    
    // Swap owner deploys
    let swap_admin = Addr::unchecked("swap_deployer");
    // cw721_owner owns the cw721
    let cw721_owner = Addr::unchecked("original_owner");
    // arch_owner owns ARCH
    let arch_owner = Addr::unchecked("arch_owner");

    // cw721_owner creates the cw721
    let nft = create_cw721(&mut app, &cw721_owner);
    
    // swap_admin creates the swap contract 
    let swap = create_swap(&mut app, &swap_admin);
    let swap_inst = swap.clone();
    
    // Mint native to `arch_owner`
    mint_native(
        &mut app,
        arch_owner.to_string(),
        Uint128::from(10000000000000000000_u128), // 10 ARCH as aarch
    );

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

    // Create a SwapMsg for creating / finishing a swap
    let creation_msg = SwapMsg {
        id: "firstswap".to_string(),
        cw721: nft.clone(),
        payment_token: None,
        token_id: token_id.clone(),    
        expires: Expiration::from(cw20::Expiration::AtHeight(384798573487439743)),
        price: Uint128::from(1000000000000000000_u128), // 1 ARCH as aarch
        swap_type: SwapType::Sale,
    };
    let finish_msg = creation_msg.clone();

    // Seller (cw721_owner) must approve the swap contract to spend their NFT
    let nft_approve_msg = Cw721ExecuteMsg::Approve::<Extension> {
        spender: swap.to_string(),
        token_id: token_id.clone(),
        expires: None,
    };
    let _res = app
        .execute_contract(cw721_owner.clone(), nft.clone(), &nft_approve_msg, &[])
        .unwrap();

    // cw721 seller (cw721_owner) creates a swap
    let _res = app
        .execute_contract(cw721_owner.clone(), swap_inst.clone(), &ExecuteMsg::Create(creation_msg), &[])
        .unwrap();

    // Buyer purchases cw721, paying 10 ARCH and consuming the swap
    let _res = app
        .execute_contract(
            arch_owner.clone(), 
            swap_inst.clone(), 
            &ExecuteMsg::Finish(finish_msg), 
            &[Coin {
                denom: String::from(DENOM),
                amount: Uint128::from(10000000000000000000_u128)
            }]
        )
        .unwrap();

    // arch_owner has received the NFT
    let owner_query: OwnerOfResponse = query(
        &mut app,
        nft.clone(),
        Cw721QueryMsg::OwnerOf {
            token_id: token_id, 
            include_expired: None
        }
    ).unwrap();

    // cw721_owner has received the ARCH amount
    let balance_query: Coin = bank_query(&mut app, &cw721_owner);

    assert_eq!(owner_query.owner, arch_owner);
    assert_eq!(balance_query.amount, Uint128::from(10000000000000000000_u128));
}

// Over paying will fail, must send exactly the required funds
// to create a SwapType::Offer for native ARCH
#[test]
fn test_overpayment_native_offer() {
    let mut app = mock_app();
    
    // Swap owner deploys
    let swap_admin = Addr::unchecked("swap_deployer");
    // cw721_owner owns the cw721
    let cw721_owner = Addr::unchecked("original_owner");
    // arch_owner owns ARCH
    let arch_owner = Addr::unchecked("arch_owner");

    // cw721_owner creates the cw721
    let nft = create_cw721(&mut app, &cw721_owner);
    
    // swap_admin creates the swap contract 
    let swap = create_swap(&mut app, &swap_admin);
    let swap_inst = swap.clone();
    
    // Mint native to `arch_owner`
    mint_native(
        &mut app,
        arch_owner.to_string(),
        Uint128::from(10000000000000000000_u128), // 10 ARCH as aarch
    );

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

    // Bidding buyer creates an offer
    let swap_id: String = "firstswap".to_string();
    let creation_msg = SwapMsg {
        id: swap_id.clone(),
        cw721: nft.clone(),
        payment_token: None,
        token_id: token_id.clone(),    
        expires: Expiration::from(cw20::Expiration::AtHeight(384798573487439743)),
        price: Uint128::from(5000000000000000000_u128), // 5 ARCH as aarch
        swap_type: SwapType::Offer,
    };

    // Sending more funds than the value of price in creation_msg
    // causes the tx to fail with ContractError::ExactFunds
    assert!(
        app.execute_contract(
            arch_owner.clone(), 
            swap_inst.clone(), 
            &ExecuteMsg::Create(creation_msg), 
            &[Coin {
                denom: String::from(DENOM),
                amount: Uint128::from(9000000000000000000_u128)
            }]
        ).is_err()
    );
}

// cw721 buyer increases payment allowance too high
// but correct payment for swap is still enforced.
// (True for both SwapType::Sale and SwapType::Offer)
#[test]
fn test_overpayment_cw20() {
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
    let swap = create_swap(&mut app, &swap_admin);
    let swap_inst = swap.clone();
    
    // cw20_owner creates a cw20 coin
    let cw20 = create_cw20(
        &mut app,
        &cw20_owner,
        "testcw".to_string(),
        "tscw".to_string(),
        Uint128::from(1000000_u32)
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

    // Create a SwapMsg for creating / finishing a swap
    let creation_msg = SwapMsg {
        id: "firstswap".to_string(),
        cw721: nft.clone(),
        payment_token: Some(Addr::unchecked(cw20.clone())),
        token_id: token_id.clone(),    
        expires: Expiration::from(cw20::Expiration::AtHeight(384798573487439743)),
        price: Uint128::from(100000_u32),
        swap_type:SwapType::Offer,
    };
    let finish_msg = creation_msg.clone();

    // Seller (cw721_owner) must approve the swap contract to spend their NFT
    let nft_approve_msg = Cw721ExecuteMsg::Approve::<Extension> {
        spender: swap.to_string(),
        token_id: token_id.clone(),
        expires: None,
    };
    let _res = app
        .execute_contract(cw721_owner.clone(), nft.clone(), &nft_approve_msg, &[])
        .unwrap();

    // cw721 seller (cw721_owner) creates a swap
    let _res = app
        .execute_contract(cw20_owner.clone(), swap_inst.clone(), &ExecuteMsg::Create(creation_msg), &[])
        .unwrap();

    // cw721 buyer (cw20_owner) allows swap contract to spend too many cw20s
    let cw20_approve_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: swap.to_string(),
        amount:  Uint128::from(900000_u32),
        expires: None,
    };
    let _res = app
        .execute_contract(cw20_owner.clone(), cw20, &cw20_approve_msg, &[])
        .unwrap();

    // Buyer purchases cw721, consuming the swap
    let _res = app
        .execute_contract(cw721_owner.clone(), swap_inst.clone(), &ExecuteMsg::Finish(finish_msg), &[])
        .unwrap();

    // cw20_owner has received the NFT
    let owner_query: OwnerOfResponse = query(
        &mut app,nft.clone(),
        Cw721QueryMsg::OwnerOf {
            token_id: token_id, 
            include_expired: None
        }
    ).unwrap();

    // cw721_owner has still received the correct cw20 amount
    let buyer_balance_query: BalanceResponse = query(
        &mut app,
        cw20_inst.clone(),
        Cw20QueryMsg::Balance {
            address: cw721_owner.to_string()
        }
    ).unwrap();

    // swap contract has spent correct cw20 amount from cw20_owner's balance
    let seller_balance_query: BalanceResponse = query(
        &mut app,
        cw20_inst,
        Cw20QueryMsg::Balance {
            address: cw20_owner.to_string()
        }
    ).unwrap();

    assert_eq!(owner_query.owner, cw20_owner);
    assert_eq!(buyer_balance_query.balance, Uint128::from(100000_u32));
    assert_eq!(seller_balance_query.balance, Uint128::from(900000_u32));
}