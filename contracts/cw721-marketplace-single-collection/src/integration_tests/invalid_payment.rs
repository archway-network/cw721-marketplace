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
    ExecuteMsg, FinishSwapMsg, SwapMsg,
};
use crate::state::{SwapType};

static DENOM: &str = "aarch";

// cw721 buyer must send correct ARCH amount
#[test]
fn test_invalid_payment_native() {
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
    let swap = create_swap(&mut app, &swap_admin, nft.clone());
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
        payment_token: None,
        token_id: token_id.clone(),    
        expires: Expiration::from(cw20::Expiration::AtHeight(384798573487439743)),
        price: Uint128::from(5000000000000000000_u128), // 5 ARCH as aarch
        swap_type: SwapType::Sale,
    };
    let finish_msg = FinishSwapMsg {
        id: creation_msg.id.clone(),
    };

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

    // Buyer attempts to purchase cw721, under paying 1 ARCH
    assert!(app.execute_contract(
        arch_owner.clone(), 
        swap_inst.clone(), 
        &ExecuteMsg::Finish(finish_msg), 
        &[Coin {
            denom: String::from(DENOM),
            amount: Uint128::from(1000000000000000000_u128)
        }]
    )
    .is_err());

    // cw721_owner has retained the NFT
    let owner_query: OwnerOfResponse = query(
        &mut app,
        nft.clone(),
        Cw721QueryMsg::OwnerOf {
            token_id: token_id, 
            include_expired: None
        }
    ).unwrap();

    // cw721_owner has not received the ARCH amount
    let cw721_owner_balance: Coin = bank_query(&mut app, &cw721_owner);
    // dbg!(cw721_owner_balance.amount);

    // arch_owner has retained their original balance (minus gas fees)
    let arch_owner_balance: Coin = bank_query(&mut app, &cw721_owner);
    // dbg!(arch_owner_balance.amount);

    assert_eq!(cw721_owner_balance.amount.u128(), 0);
    assert_eq!(arch_owner_balance.amount.u128(), 0);
    assert_eq!(owner_query.owner, cw721_owner);
}

// ARCH offers must send the correct ARCH payment
#[test]
fn test_invalid_payment_native_offer() {
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
    let swap = create_swap(&mut app, &swap_admin, nft.clone());
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

    // Bidding buyer creates an offer (with an invalid payment)
    let swap_id: String = "firstswap".to_string();
    let creation_msg = SwapMsg {
        id: swap_id.clone(),
        payment_token: None,
        token_id: token_id.clone(),    
        expires: Expiration::from(cw20::Expiration::AtHeight(384798573487439743)),
        price: Uint128::from(9000000000000000000_u128), // 9 ARCH as aarch
        swap_type: SwapType::Offer,
    };

    // Invalid payment must err
    assert!(
        app.execute_contract(
            arch_owner.clone(), 
            swap_inst.clone(), 
            &ExecuteMsg::Create(creation_msg), 
            &[Coin {
                denom: String::from(DENOM),
                amount: Uint128::from(8000000000000000000_u128)
            }]
        ).is_err()
    );

    // cw721_owner has not transferred the NFT
    let owner_query: OwnerOfResponse = query(
        &mut app,nft.clone(),
        Cw721QueryMsg::OwnerOf {
            token_id: token_id, 
            include_expired: None
        }
    ).unwrap();
    assert_eq!(owner_query.owner, cw721_owner);

    // Bidding buyer's account has not been debited
    let arch_owner_balance: Coin = bank_query(&mut app, &arch_owner);
    assert_eq!(arch_owner_balance.amount, Uint128::from(10000000000000000000_u128));
}

// Sales cannot be consumed with invalid payment parameters
// cw721 buyer must approve (and own) sufficient cw20 amount
#[test]
fn test_invalid_payment_cw20() {
    let mut app = mock_app();
    
    // Swap owner deploys
    let swap_admin = Addr::unchecked("swap_deployer");
    // cw721_owner owns the cw721
    let cw721_owner = Addr::unchecked("original_owner");
    // cw20_owner owns the cw20
    let cw20_owner = Addr::unchecked("cw20_owner");
    // random has no cw20 or cw721 tokens
    let random = Addr::unchecked("owns_nothing");

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
        Uint128::from(100000_u32)
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
        payment_token: Some(Addr::unchecked(cw20.clone())),
        token_id: token_id.clone(),    
        expires: Expiration::from(cw20::Expiration::AtHeight(384798573487439743)),
        price: Uint128::from(100000_u32),
        swap_type:SwapType::Offer,
    };
    let finish_msg = FinishSwapMsg {
        id: creation_msg.id.clone(),
    };

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

    // cw20_owner does not approve enough funds
    let cw20_approve_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: swap.to_string(),
        amount:  Uint128::from(10000_u32),
        expires: None,
    };
    let _res = app
        .execute_contract(cw20_owner.clone(), cw20, &cw20_approve_msg, &[])
        .unwrap();

    // cw20's purchase fails
    assert!(
        app.execute_contract(
            cw20_owner.clone(), 
            swap_inst.clone(), 
            &ExecuteMsg::Finish(finish_msg.clone()), 
            &[]
        ).is_err()
    );

    // random has no cw20, their purchase fails
    assert!(
        app.execute_contract(
            random.clone(), 
            swap_inst.clone(), 
            &ExecuteMsg::Finish(finish_msg), 
            &[]
        ).is_err()
    );
}

// cw20 offers cannot be consumed with invalid approvals, or 
// insufficient funds. cw20 owners must approve marketplace
// to spend all funds required by the swap price
#[test]
fn test_invalid_payment_cw20_offer() {
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
        Uint128::from(100000_u32)
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

    // Bidding buyer (cw20_owner) does not approve enough funds
    let cw20_approve_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: swap.to_string(),
        amount:  Uint128::from(10000_u32),
        expires: None,
    };
    let _res = app
        .execute_contract(cw20_owner.clone(), cw20.clone(), &cw20_approve_msg, &[])
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
        .execute_contract(cw20_owner.clone(), swap_inst.clone(), &ExecuteMsg::Create(creation_msg), &[])
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

    // cw721_owner accepts the cw20 buyer's offer but the swap must fail (invalid payment)
    assert!(
        app.execute_contract(
            cw721_owner.clone(), 
            swap_inst.clone(), 
            &ExecuteMsg::Finish(finish_msg), 
            &[]
        ).is_err()
    );

    // cw721_owner has not transferred the NFT
    let owner_query: OwnerOfResponse = query(
        &mut app,nft.clone(),
        Cw721QueryMsg::OwnerOf {
            token_id: token_id, 
            include_expired: None
        }
    ).unwrap();
    assert_eq!(owner_query.owner, cw721_owner);

    // cw20_owner has not transferred the cw20 amount
    let balance_query: BalanceResponse = query(
        &mut app,
        cw20_inst,
        Cw20QueryMsg::Balance {
            address: cw20_owner.to_string()
        }
    ).unwrap();
    assert_eq!(balance_query.balance, Uint128::from(100000_u32));
}