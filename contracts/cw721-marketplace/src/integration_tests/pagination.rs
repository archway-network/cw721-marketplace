#![cfg(test)]
use cosmwasm_std::{
    Addr, Uint128,
};
use cw_multi_test::Executor;

use cw20::Expiration;
use cw721_base::{
    msg::ExecuteMsg as Cw721ExecuteMsg, Extension, MintMsg,
};

use crate::integration_tests::util::{
    create_cw721, create_swap, has_unique_elements, mock_app, query,
};
use crate::msg::{
    ExecuteMsg, ListResponse, QueryMsg, SwapMsg,
};
use crate::state::SwapType;
use crate::query::PageResult;

// Listing swaps and querying filter entry points must be enumerable,
// and must return correct results, totals, and page for all page sizes
#[test]
fn test_pagination() {
    let mut app = mock_app();
    
    // Swap owner deploys
    let swap_admin = Addr::unchecked("swap_deployer");

    // cw721_owner owns cw721 tokens
    let cw721_owner = Addr::unchecked("cw721_owner");

    // cw721_owner creates cw721 token contract
    let nft = create_cw721(&mut app, &cw721_owner);

    // swap_admin creates the swap contract 
    let swap = create_swap(&mut app, &swap_admin);
    let swap_inst = swap.clone();

    // cw721_owner mints 15 cw721 tokens 
    let token_ids = vec![
        // Page 1 of 3
        "token1".to_string(),"token2".to_string(),"token3".to_string(),"token4".to_string(),"token5".to_string(),
        // Page 2 of 3
        "token6".to_string(),"token7".to_string(),"token8".to_string(),"token9".to_string(),"token10".to_string(),
        // Page 3 of 3
        "token11".to_string(),"token12".to_string(),"token13".to_string(),"token14".to_string(),"token15".to_string(),
    ];
    let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();

    // Mint 15 tokens and create a swap for each
    for token_id in token_ids.iter() {
        // Mint msg
        let mint_msg = Cw721ExecuteMsg::Mint(MintMsg::<Extension> {
            token_id: token_id.clone(),
            owner: cw721_owner.to_string(),
            token_uri: Some(token_uri.clone()),
            extension: None,
        });
        // Do minting
        let _res = app
            .execute_contract(cw721_owner.clone(), nft.clone(), &mint_msg, &[])
            .unwrap();

        // Approval msg
        let nft_approve_msg = Cw721ExecuteMsg::Approve::<Extension> {
            spender: swap.to_string(),
            token_id: token_id.clone(),
            expires: None,
        };
        // Do approve marketplace as spender
        let _res = app
            .execute_contract(cw721_owner.clone(), nft.clone(), &nft_approve_msg, &[])
            .unwrap();

        // Swap msg
        let creation_msg = SwapMsg {
            id: token_id.clone(),
            cw721: nft.clone(),
            payment_token: None,
            token_id: token_id.clone(),    
            expires: Expiration::from(cw20::Expiration::AtHeight(384798573487439743)),
            price: Uint128::from(1000000000000000000_u128), // 1 ARCH as aarch
            swap_type: SwapType::Sale,
        };
        // Create swap listing
        let _res = app
            .execute_contract(cw721_owner.clone(), swap_inst.clone(), &ExecuteMsg::Create(creation_msg), &[])
            .unwrap();
    }

    // Query List entry point for 3 pages
    // Paging size
    let limit: u32 = 5;
    // Page 1
    let page_1: ListResponse = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::List {
            start_after: None,
            limit: Some(limit.clone()),
        }
    ).unwrap();
    // Page 2
    let page_2: ListResponse = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::List {
            start_after: Some(page_1.swaps[4].clone()),
            limit: Some(limit.clone()),
        }
    ).unwrap();
    // Page 3
    let page_3: ListResponse = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::List {
            start_after: Some(page_2.swaps[4].clone()),
            limit: Some(limit.clone()),
        }
    ).unwrap();
    
    // Paginated results must not have duplicates
    let mut all_res = page_1.swaps.clone();
    all_res.append(&mut page_2.swaps.clone());
    all_res.append(&mut page_3.swaps.clone());
    assert!(has_unique_elements(all_res));

    // Paginated results must each have a size equal to `limit`
    assert_eq!(page_1.swaps.len(), 5);
    assert_eq!(page_2.swaps.len(), 5);
    assert_eq!(page_3.swaps.len(), 5);

    // Query GetListings entry point for 2 pages
    // Page 1
    let page_1b: PageResult = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::GetListings {
            page: None,
            limit: None,
        }
    ).unwrap();
    // Page 2
    let page_2b: PageResult = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::GetListings {
            page: Some(1_u32),
            limit: None,
        }
    ).unwrap();

    // Paginated results must have correct page sizes
    assert_eq!(page_1b.swaps.len(), 10);
    assert_eq!(page_2b.swaps.len(), 5);
    // Paginated results must include correct total
    assert_eq!(page_1b.total, 15);
    assert_eq!(page_2b.total, 15);

    // Paginated results must not have duplicates
    let mut all_res_b = page_1b.swaps.clone();
    all_res_b.append(&mut page_2b.swaps.clone());
    let mut token_ids_b: Vec<String> = vec![];
    for swap in all_res_b.iter() {
        token_ids_b.push(swap.clone().token_id);
    }
    assert!(has_unique_elements(token_ids_b));

    // Query SwapsOf entry point for 2 pages
    // Page 1
    let page_1c: PageResult = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::SwapsOf {
            address: cw721_owner.clone(),
            swap_type: Some(SwapType::Sale),
            cw721: None,
            page: None,
            limit: None,
        }
    ).unwrap();
    // Page 2
    let page_2c: PageResult = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::SwapsOf {
            address: cw721_owner.clone(),
            swap_type: Some(SwapType::Sale),
            cw721: None,
            page: Some(1_u32),
            limit: None,
        }
    ).unwrap();

    // Paginated results must have correct page sizes
    assert_eq!(page_1c.swaps.len(), 10);
    assert_eq!(page_2c.swaps.len(), 5);
    // Paginated results must include correct total
    assert_eq!(page_1c.total, 15);
    assert_eq!(page_2c.total, 15);

    // Paginated results must not have duplicates
    let mut all_res_c = page_1c.swaps.clone();
    all_res_c.append(&mut page_2c.swaps.clone());
    let mut token_ids_c: Vec<String> = vec![];
    for swap in all_res_c.iter() {
        token_ids_c.push(swap.clone().token_id);
    }
    assert!(has_unique_elements(token_ids_c));

    // Query SwapsByPrice entry point for 2 pages
    // Page 1
    let page_1d: PageResult = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::SwapsByPrice {
            min: Some(Uint128::from(0_u128)),
            max: Some(Uint128::from(1000000000000000000_u128)),
            swap_type: Some(SwapType::Sale),
            cw721: None,
            page: None,
            limit: None,
        }
    ).unwrap();
    // Page 2
    let page_2d: PageResult = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::SwapsByPrice {
            min: Some(Uint128::from(0_u128)),
            max: Some(Uint128::from(1000000000000000000_u128)),
            swap_type: Some(SwapType::Sale),
            cw721: None,
            page: Some(1_u32),
            limit: None,
        }
    ).unwrap();

    // Paginated results must have correct page sizes
    assert_eq!(page_1d.swaps.len(), 10);
    assert_eq!(page_2d.swaps.len(), 5);
    // Paginated results must include correct total
    assert_eq!(page_1d.total, 15);
    assert_eq!(page_2d.total, 15);

    // Paginated results must not have duplicates
    let mut all_res_d = page_1d.swaps.clone();
    all_res_d.append(&mut page_2d.swaps.clone());
    let mut token_ids_d: Vec<String> = vec![];
    for swap in all_res_d.iter() {
        token_ids_d.push(swap.clone().token_id);
    }
    assert!(has_unique_elements(token_ids_d));

    // Query SwapsByDenom entry point for 2 pages
    // Page 1
    let page_1e: PageResult = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::SwapsByDenom {
            payment_token: None,
            swap_type: Some(SwapType::Sale),
            cw721: None,
            page: None,
            limit: None,
        }
    ).unwrap();
    // Page 2
    let page_2e: PageResult = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::SwapsByDenom {
            payment_token: None,
            swap_type: Some(SwapType::Sale),
            cw721: None,
            page: Some(1_u32),
            limit: None,
        }
    ).unwrap();

    // Paginated results must have correct page sizes
    assert_eq!(page_1e.swaps.len(), 10);
    assert_eq!(page_2e.swaps.len(), 5);
    // Paginated results must include correct total
    assert_eq!(page_1e.total, 15);
    assert_eq!(page_2e.total, 15);

    // Paginated results must not have duplicates
    let mut all_res_e = page_1e.swaps.clone();
    all_res_e.append(&mut page_2e.swaps.clone());
    let mut token_ids_e: Vec<String> = vec![];
    for swap in all_res_e.iter() {
        token_ids_e.push(swap.clone().token_id);
    }
    assert!(has_unique_elements(token_ids_e));


    // Query SwapsByPaymentType entry point for 2 pages
    // Page 1
    let page_1f: PageResult = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::SwapsByPaymentType {
            cw20: false,
            swap_type: Some(SwapType::Sale),
            cw721: None,
            page: None,
            limit: None,
        }
    ).unwrap();
    // Page 2
    let page_2f: PageResult = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::SwapsByPaymentType {
            cw20: false,
            swap_type: Some(SwapType::Sale),
            cw721: None,
            page: Some(1_u32),
            limit: None,
        }
    ).unwrap();

    // Paginated results must have correct page sizes
    assert_eq!(page_1f.swaps.len(), 10);
    assert_eq!(page_2f.swaps.len(), 5);
    // Paginated results must include correct total
    assert_eq!(page_1f.total, 15);
    assert_eq!(page_2f.total, 15);

    // Paginated results must not have duplicates
    let mut all_res_f = page_1f.swaps.clone();
    all_res_f.append(&mut page_2f.swaps.clone());
    let mut token_ids_f: Vec<String> = vec![];
    for swap in all_res_f.iter() {
        token_ids_f.push(swap.clone().token_id);
    }
    assert!(has_unique_elements(token_ids_f));

    // Query ListingsOfToken entry point (All Listings)
    let listings_of_token_a: PageResult = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::ListingsOfToken {
            token_id: "token10".to_string(),
            cw721: nft.clone(),
            swap_type: None, // All Listings
            page: None,
            limit: None,
        }
    ).unwrap();
    // 1 Result
    assert_eq!(listings_of_token_a.swaps.len(), 1);

    // Query ListingsOfToken entry point (Sales)
    let listings_of_token_b: PageResult = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::ListingsOfToken {
            token_id: "token10".to_string(),
            cw721: nft.clone(),
            swap_type: Some(SwapType::Sale), // Sale Listings
            page: None,
            limit: None,
        }
    ).unwrap();
    // 1 Result
    assert_eq!(listings_of_token_b.swaps.len(), 1);

    // Query ListingsOfToken entry point (Offers)
    let listings_of_token_c: PageResult = query(
        &mut app,
        swap_inst.clone(),
        QueryMsg::ListingsOfToken {
            token_id: "token10".to_string(),
            cw721: nft,
            swap_type: Some(SwapType::Offer), // Offer Listings
            page: None,
            limit: None,
        }
    ).unwrap();
    // 0 Results
    assert_eq!(listings_of_token_c.swaps.len(), 0);
}