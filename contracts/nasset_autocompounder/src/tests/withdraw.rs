use crate::msg::{Cw20HookMsg, ExecuteMsg};

use super::sdk::{Sdk, AUTO_NASSET_TOKEN_ADDR, NASSET_TOKEN_ADDR};

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{to_binary, CosmosMsg, StdError, SubMsg, Uint128, WasmMsg};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

#[test]
fn fail_to_withdraw_wrong_cw20() {
    let mut sdk = Sdk::init();
    let sender_addr = "some_sender";

    let cw20_withdraw_msg = Cw20ReceiveMsg {
        sender: sender_addr.to_string(),
        amount: Uint128::from(256u64),
        msg: to_binary(&Cw20HookMsg::Withdraw {}).unwrap(),
    };

    let info = mock_info("some_random_addr", &[]);
    let res = crate::contract::execute(
        sdk.deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(cw20_withdraw_msg),
    );

    assert!(res.is_err());
    let error_value = res.err().unwrap();
    assert_eq!(StdError::generic_err("unauthorized"), error_value);
}

#[test]
fn withdraw_nasset() {
    let mut sdk = Sdk::init();

    let initial_auto_nasset_supply: Uint256 = 10_000_000_000u128.into();
    //first farmer come
    let user_1_address = "addr9999".to_string();
    let withdraw_1_amount: Uint256 = 2_000_000_000u128.into();
    let two: Uint256 = Uint256::from(2u128);
    {
        sdk.set_auto_nasset_supply(initial_auto_nasset_supply);
        sdk.set_nasset_balance(initial_auto_nasset_supply * two);

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
                        amount: (withdraw_1_amount * two).into(), // cause nasset_balance is twice auto_nasset_balance
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

    //second farmer comes
    let user_2_address = "addr6666".to_string();
    let withdraw_2_amount: Uint256 = 6_000_000_000u128.into();
    {
        sdk.set_auto_nasset_supply(initial_auto_nasset_supply - withdraw_1_amount);
        sdk.set_nasset_balance(initial_auto_nasset_supply * two - withdraw_1_amount * two);
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
                        amount: (withdraw_2_amount * two).into(), // cause nasset_balance is twice auto_nasset_balance
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
#[allow(non_snake_case)]
fn deposit__nasset_increased__withdraw() {
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

    //increase nasset_amount
    let new_nasset_amount = deposit_1_amount + Uint256::from(1_000_000_000u128);
    sdk.set_nasset_balance(new_nasset_amount);

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

#[test]
fn withdraw_with_profit() {
    let mut sdk = Sdk::init();

    let initial_nasset_supply: Uint256 = 10_000_000_000u128.into();
    //first farmer come
    let user_1_address = "addr9999".to_string();
    let withdraw_1_amount: Uint256 = 2_000_000_000u128.into();
    let profit: Uint256 = Uint256::from(1_000_000_000u128);
    {
        sdk.set_auto_nasset_supply(withdraw_1_amount);
        sdk.set_nasset_balance(initial_nasset_supply);

        let response = sdk
            .user_withdraw(&user_1_address, withdraw_1_amount.into(), profit)
            .unwrap();

        assert_eq!(
            response.messages,
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: user_1_address.clone(),
                        amount: (initial_nasset_supply + profit).into(),
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
}

#[test]
fn withdraw_nasset_without_rewards() {
    let mut sdk = Sdk::init();

    let initial_auto_nasset_supply: Uint256 = 10_000_000_000u128.into();
    //first farmer come
    let user_1_address = "addr9999".to_string();
    let withdraw_1_amount: Uint256 = 2_000_000_000u128.into();
    let two: Uint256 = Uint256::from(2u128);
    {
        sdk.set_auto_nasset_supply(initial_auto_nasset_supply);
        sdk.set_nasset_balance(initial_auto_nasset_supply * two);

        let response = sdk
            .user_withdraw_without_nasset_rewards(&user_1_address, withdraw_1_amount.into())
            .unwrap();

        assert_eq!(
            response.messages,
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: user_1_address.clone(),
                        amount: (withdraw_1_amount * two).into(), // cause nasset_balance is twice auto_nasset_balance
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

    //second farmer comes
    let user_2_address = "addr6666".to_string();
    let withdraw_2_amount: Uint256 = 6_000_000_000u128.into();
    {
        sdk.set_auto_nasset_supply(initial_auto_nasset_supply - withdraw_1_amount);
        sdk.set_nasset_balance(initial_auto_nasset_supply * two - withdraw_1_amount * two);
        let response = sdk
            .user_withdraw_without_nasset_rewards(&user_2_address, withdraw_2_amount.into())
            .unwrap();

        assert_eq!(
            response.messages,
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: user_2_address.clone(),
                        amount: (withdraw_2_amount * two).into(), // cause nasset_balance is twice auto_nasset_balance
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
