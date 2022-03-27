use crate::msg::{Cw20HookMsg, ExecuteMsg};

use super::sdk::{Sdk, AUTO_NASSET_TOKEN_ADDR};

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{to_binary, CosmosMsg, StdError, SubMsg, Uint128, WasmMsg};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

#[test]
fn fail_to_deposit_wrong_cw20() {
    let mut sdk = Sdk::init();
    let sender_addr = "some_sender";

    let cw20_deposit_msg = Cw20ReceiveMsg {
        sender: sender_addr.to_string(),
        amount: Uint128::from(256u64),
        msg: to_binary(&Cw20HookMsg::Deposit {}).unwrap(),
    };

    let info = mock_info("some_random_addr", &[]);
    let res = crate::contract::execute(
        sdk.deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(cw20_deposit_msg),
    );

    assert!(res.is_err());
    let error_value = res.err().unwrap();
    assert_eq!(StdError::generic_err("unauthorized"), error_value);
}

#[test]
fn deposit_nasset() {
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

    //second farmer come
    let user_2_address = "addr6666".to_string();
    let deposit_2_amount: Uint256 = 6_000_000_000u128.into();
    {
        sdk.set_auto_nasset_supply(deposit_1_amount);
        sdk.set_nasset_balance(deposit_2_amount + deposit_1_amount);
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
                    amount: deposit_2_amount.into(),
                })
                .unwrap(),
                funds: vec![],
            })),]
        );
    }
}

#[test]
fn deposit_nasset_after_someone_transfer_some_nassets_directly_to_contract() {
    let mut sdk = Sdk::init();

    //first farmer come
    let user_address = "addr9999".to_string();
    let deposit_amount: Uint256 = 2_000_000_000u128.into();
    let nasset_directly_tranfered_amount: Uint256 = 10_000_000_000u128.into();
    let total_nasset_amount = deposit_amount + nasset_directly_tranfered_amount;
    {
        // -= USER SEND nAsset tokens to autocompounder =-
        sdk.set_auto_nasset_supply(Uint256::zero());
        sdk.set_nasset_balance(total_nasset_amount);

        let response = sdk
            .user_deposit(&user_address, deposit_amount.into())
            .unwrap();

        assert_eq!(
            response.messages,
            vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: AUTO_NASSET_TOKEN_ADDR.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: user_address.clone(),
                    amount: deposit_amount.into(),
                })
                .unwrap(),
                funds: vec![],
            })),]
        );
    }
}
