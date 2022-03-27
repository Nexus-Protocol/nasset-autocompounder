use super::sdk::{Sdk, AUTO_NASSET_TOKEN_ADDR, NASSET_TOKEN_ADDR};

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{to_binary, CosmosMsg, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

#[test]
fn withdraw_compound_withdraw() {
    let mut sdk = Sdk::init();

    let initial_nasset_supply: Uint256 = 10u128.into();
    //first farmer come
    let user_1_address = "addr9999".to_string();
    let withdraw_1_amount: Uint256 = 2u128.into();
    {
        sdk.set_auto_nasset_supply(initial_nasset_supply);
        sdk.set_nasset_balance(initial_nasset_supply);

        let response = sdk
            .user_withdraw(&user_1_address, withdraw_1_amount.into(), Uint256::zero())
            .unwrap();

        assert_eq!(
            response.messages,
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: user_1_address.clone(),
                        amount: withdraw_1_amount.into(),
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: AUTO_NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Burn {
                        amount: withdraw_1_amount.into(),
                    })
                    .unwrap(),
                    funds: vec![],
                })),
            ]
        );
    }
    sdk.set_nasset_balance(initial_nasset_supply - withdraw_1_amount);

    // COMPOUND
    let compound_profit: Uint256 = Uint256::from(8u128);
    sdk.user_send_compound(compound_profit).unwrap();

    //second farmer comes to withdraw
    let user_2_address = "addr6666".to_string();
    let withdraw_2_amount: Uint256 = 2u128.into();
    {
        sdk.set_auto_nasset_supply(initial_nasset_supply - withdraw_1_amount);
        let response = sdk
            .user_withdraw(&user_2_address, withdraw_2_amount.into(), Uint256::zero())
            .unwrap();

        assert_eq!(
            response.messages,
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: user_2_address.clone(),
                        // withdrawing 2 (total 8), means 1/4 share
                        // total_nasset = 16
                        // 16 / 4 = 4
                        amount: Uint128::from(4u128),
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: AUTO_NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Burn {
                        amount: withdraw_2_amount.into(),
                    })
                    .unwrap(),
                    funds: vec![],
                })),
            ]
        );
    }
}

#[test]
fn withdraw_compound_deposit() {
    let mut sdk = Sdk::init();

    let initial_nasset_supply: Uint256 = 10u128.into();
    //first farmer come
    let user_1_address = "addr9999".to_string();
    let withdraw_1_amount: Uint256 = 2u128.into();
    {
        sdk.set_auto_nasset_supply(initial_nasset_supply);
        sdk.set_nasset_balance(initial_nasset_supply);

        let response = sdk
            .user_withdraw(&user_1_address, withdraw_1_amount.into(), Uint256::zero())
            .unwrap();

        assert_eq!(
            response.messages,
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: user_1_address.clone(),
                        amount: withdraw_1_amount.into(),
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: AUTO_NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Burn {
                        amount: withdraw_1_amount.into(),
                    })
                    .unwrap(),
                    funds: vec![],
                })),
            ]
        );
    }
    sdk.set_nasset_balance(initial_nasset_supply - withdraw_1_amount);

    // COMPOUND
    let compound_profit: Uint256 = Uint256::from(8u128);
    sdk.user_send_compound(compound_profit).unwrap();

    //second farmer come
    let deposit_2_amount: Uint256 = 4u128.into();
    sdk.increase_nasset_balance(deposit_2_amount);

    let user_2_address = "addr6666".to_string();
    {
        sdk.set_auto_nasset_supply(initial_nasset_supply - withdraw_1_amount);
        // -= USER SEND nAsset tokens to autocompounder =-
        let response = sdk
            .user_deposit(&user_2_address, deposit_2_amount.into())
            .unwrap();

        assert_eq!(
            response.messages,
            vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: AUTO_NASSET_TOKEN_ADDR.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: user_2_address.clone(),
                    // anasset_supply * deposit_amount / (nasset_balance - deposit_amount)
                    // 8 * 4 / (20 - 4)
                    amount: 2u128.into(),
                })
                .unwrap(),
                funds: vec![],
            })),]
        );
    }
}

#[test]
fn deposit_compound_deposit() {
    let mut sdk = Sdk::init();

    //first farmer come
    let user_1_address = "addr9999".to_string();
    let deposit_1_amount: Uint256 = 7_000_000_000u128.into();
    {
        // -= USER SEND nAsset tokens to autocompounder =-
        sdk.set_auto_nasset_supply(Uint256::zero());
        sdk.set_nasset_balance(deposit_1_amount);

        let response = sdk
            .user_deposit(&user_1_address, deposit_1_amount.into())
            .unwrap();

        assert_eq!(
            response.messages,
            vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: AUTO_NASSET_TOKEN_ADDR.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: user_1_address.clone(),
                    amount: deposit_1_amount.into(),
                })
                .unwrap(),
                funds: vec![],
            })),]
        );
    }

    // COMPOUND
    let compound_profit = Uint256::from(1_000_000_000u128);
    sdk.user_send_compound(compound_profit).unwrap();

    //second farmer come
    let deposit_2_amount: Uint256 = 6_000_000_000u128.into();
    sdk.increase_nasset_balance(deposit_2_amount);

    let user_2_address = "addr6666".to_string();
    {
        sdk.set_auto_nasset_supply(deposit_1_amount);
        // -= USER SEND nAsset tokens to autocompounder =-
        let response = sdk
            .user_deposit(&user_2_address, deposit_2_amount.into())
            .unwrap();

        assert_eq!(
            response.messages,
            vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: AUTO_NASSET_TOKEN_ADDR.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: user_2_address.clone(),
                    // anasset_supply * deposit_amount / (nasset_balance - deposit_amount)
                    // 7B * 6B / (14B - 6B)
                    amount: 5_250_000_000u128.into(),
                })
                .unwrap(),
                funds: vec![],
            })),]
        );
    }
}

#[test]
fn deposit_compound_withdraw() {
    let mut sdk = Sdk::init();

    //first farmer come
    let user_1_address = "addr9999".to_string();
    let deposit_1_amount: Uint256 = 2_000_000_000u128.into();
    {
        // -= USER SEND nAsset tokens to autocompounder =-
        sdk.set_auto_nasset_supply(Uint256::zero());
        sdk.set_nasset_balance(deposit_1_amount);

        let response = sdk
            .user_deposit(&user_1_address, deposit_1_amount.into())
            .unwrap();

        assert_eq!(
            response.messages,
            vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: AUTO_NASSET_TOKEN_ADDR.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: user_1_address.clone(),
                    amount: deposit_1_amount.into(),
                })
                .unwrap(),
                funds: vec![],
            })),]
        );
    }

    // COMPOUND
    let compound_profit = Uint256::from(1_000_000_000u128);
    sdk.user_send_compound(compound_profit).unwrap();

    //second farmer comes to withdraw
    let user_2_address = "addr6666".to_string();
    let withdraw_2_amount: Uint256 = 1_000_000_000u128.into();
    {
        sdk.set_auto_nasset_supply(deposit_1_amount);
        let response = sdk
            .user_withdraw(&user_2_address, withdraw_2_amount.into(), Uint256::zero())
            .unwrap();

        assert_eq!(
            response.messages,
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: user_2_address.clone(),
                        amount: Uint128::from(1_500_000_000u128), // 2B + 1B(increased) / 2 (cause withdraw half of auto_nasset supply)
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: AUTO_NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Burn {
                        amount: withdraw_2_amount.into(),
                    })
                    .unwrap(),
                    funds: vec![],
                })),
            ]
        );
    }
}
