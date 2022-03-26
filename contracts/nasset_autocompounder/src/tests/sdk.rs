use crate::{
    msg::{
        AstroportCw20HookMsg, Cw20HookMsg, ExecuteMsg, InstantiateMsg, NAssetTokenRewardsAnyoneMsg,
        NAssetTokenRewardsExecuteMsg,
    },
    reply_response::MsgInstantiateContractResponse,
    state::{load_config, load_withdraw_action, Config},
    SubmsgIds,
};

use super::{mock_dependencies, WasmMockQuerier};
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    to_binary, Api, CosmosMsg, Empty, OwnedDeps, Querier, Reply, Response, StdResult, Storage,
    SubMsg, SubMsgExecutionResponse, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use protobuf::Message;
use std::collections::HashMap;
use std::iter::FromIterator;

pub const NASSET_TOKEN_ADDR: &str = "addr0001";
pub const PSI_TOKEN_ADDR: &str = "addr0002";
pub const PSI_TO_NASSET_PAIR_ADDR: &str = "addr0003";
pub const GOVERNANCE_CONTRACT_ADDR: &str = "addr0004";
pub const CW20_TOKEN_CODE_ID: u64 = 256;
pub const NASSET_TOKEN_REWARDS_ADDR: &str = "addr0005";
pub const COLLATERAL_TOKEN_SYMBOL: &str = "AVAX";
pub const AUTO_NASSET_TOKEN_ADDR: &str = "addr0006";

pub struct Sdk {
    pub deps: OwnedDeps<MockStorage, MockApi, WasmMockQuerier>,
    nasset_balance: Uint128,
    psi_balance: Uint128,
    auto_nasset_supply: Uint128,
}

impl Sdk {
    pub fn init() -> Self {
        let msg = InstantiateMsg {
            nasset_token_addr: NASSET_TOKEN_ADDR.to_string(),
            psi_token_addr: PSI_TOKEN_ADDR.to_string(),
            psi_to_nasset_pair_addr: PSI_TO_NASSET_PAIR_ADDR.to_string(),
            governance_contract_addr: GOVERNANCE_CONTRACT_ADDR.to_string(),
            cw20_token_code_id: CW20_TOKEN_CODE_ID,
            nasset_token_rewards_addr: NASSET_TOKEN_REWARDS_ADDR.to_string(),
            collateral_token_symbol: COLLATERAL_TOKEN_SYMBOL.to_string(),
        };

        let mut deps = mock_dependencies(&[]);
        Self::instantiate_nasset_autocompounder(&mut deps, msg.clone());

        Sdk {
            deps,
            nasset_balance: Uint128::zero(),
            auto_nasset_supply: Uint128::zero(),
            psi_balance: Uint128::zero(),
        }
    }

    pub fn instantiate_nasset_autocompounder<A: Storage, B: Api, C: Querier>(
        deps: &mut OwnedDeps<A, B, C>,
        init_msg: InstantiateMsg,
    ) {
        let info = mock_info("addr9999", &[]);
        crate::contract::instantiate(deps.as_mut(), mock_env(), info, init_msg.clone()).unwrap();

        // it worked, let's query the state
        let config: Config = load_config(&deps.storage).unwrap();
        assert_eq!(
            init_msg.governance_contract_addr,
            config.governance_contract.to_string()
        );
        assert_eq!(init_msg.nasset_token_addr, config.nasset_token.to_string());
        assert_eq!(config.auto_nasset_token.to_string(), "");
        assert_eq!(init_msg.psi_token_addr, config.psi_token.to_string());
        assert_eq!(
            init_msg.psi_to_nasset_pair_addr,
            config.psi_to_nasset_pair.to_string()
        );
        assert_eq!(
            init_msg.nasset_token_rewards_addr,
            config.nasset_token_rewards.to_string()
        );
        assert_eq!(
            init_msg.nasset_token_rewards_addr,
            config.nasset_token_rewards.to_string()
        );

        let withdraw_action = load_withdraw_action(&deps.storage).unwrap();
        assert!(withdraw_action.is_none());

        // ==========================================================
        // ================ Init AUTO_NASSET_TOKEN ==================
        // ==========================================================

        {
            let mut auto_nasset_token_initiate_response = MsgInstantiateContractResponse::new();
            auto_nasset_token_initiate_response
                .set_contract_address(AUTO_NASSET_TOKEN_ADDR.to_string());

            let reply_msg = Reply {
                id: SubmsgIds::InitANAsset.id(),
                result: cosmwasm_std::ContractResult::Ok(SubMsgExecutionResponse {
                    events: vec![],
                    data: Some(
                        auto_nasset_token_initiate_response
                            .write_to_bytes()
                            .unwrap()
                            .into(),
                    ),
                }),
            };

            let res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();

            assert_eq!(
                res.attributes,
                vec![
                    ("action", "auto_nasset_token_initialized"),
                    ("auto_nasset_token_addr", AUTO_NASSET_TOKEN_ADDR),
                ]
            );

            let config: Config = load_config(&deps.storage).unwrap();
            assert_eq!(config.auto_nasset_token.to_string(), AUTO_NASSET_TOKEN_ADDR);
        }
    }

    pub fn user_deposit(&mut self, address: &str, amount: Uint128) -> StdResult<Response<Empty>> {
        let cw20_deposit_msg = Cw20ReceiveMsg {
            sender: address.to_string(),
            amount,
            msg: to_binary(&Cw20HookMsg::Deposit {}).unwrap(),
        };

        let info = mock_info(NASSET_TOKEN_ADDR, &vec![]);
        crate::contract::execute(
            self.deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Receive(cw20_deposit_msg),
        )
    }

    pub fn user_withdraw(
        &mut self,
        address: &str,
        amount: Uint128,
        nasset_profit: Uint256,
    ) -> StdResult<Response<Empty>> {
        //this number means nothing
        //because we manually set nasset_profit
        let psi_claimed = Uint256::from(256_000_000u128);

        let cw20_withdraw_msg = Cw20ReceiveMsg {
            sender: address.to_string(),
            amount,
            msg: to_binary(&Cw20HookMsg::Withdraw {}).unwrap(),
        };

        let info = mock_info(AUTO_NASSET_TOKEN_ADDR, &vec![]);
        let response = crate::contract::execute(
            self.deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Receive(cw20_withdraw_msg),
        )
        .unwrap();

        assert_eq!(
            response.messages,
            vec![SubMsg::reply_on_success(
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_REWARDS_ADDR.to_string(),
                    msg: to_binary(&NAssetTokenRewardsExecuteMsg::Anyone {
                        anyone_msg: NAssetTokenRewardsAnyoneMsg::ClaimRewards { recipient: None },
                    })
                    .unwrap(),
                    funds: vec![],
                }),
                SubmsgIds::PsiClaimed.id(),
            )]
        );

        // PSI SOLD REPLY
        self.set_psi_balance(psi_claimed);
        let reply_msg = Reply {
            id: SubmsgIds::PsiClaimed.id(),
            result: cosmwasm_std::ContractResult::Ok(SubMsgExecutionResponse {
                events: vec![],
                data: None,
            }),
        };

        let res =
            crate::contract::reply(self.deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();
        assert_eq!(
            res.messages,
            vec![SubMsg::reply_on_success(
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: PSI_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        amount: psi_claimed.into(),
                        contract: PSI_TO_NASSET_PAIR_ADDR.to_string(),
                        msg: to_binary(&AstroportCw20HookMsg::Swap {
                            belief_price: None,
                            max_spread: None,
                            to: None,
                        })
                        .unwrap(),
                    })
                    .unwrap(),
                    funds: vec![],
                },),
                SubmsgIds::PsiSold.id()
            )]
        );

        // PSI SWAPPED
        self.set_psi_balance(Uint256::zero());
        self.increase_nasset_balance(nasset_profit);
        let reply_msg = Reply {
            id: SubmsgIds::PsiSold.id(),
            result: cosmwasm_std::ContractResult::Ok(SubMsgExecutionResponse {
                events: vec![],
                data: None,
            }),
        };
        let response = crate::contract::reply(self.deps.as_mut(), mock_env(), reply_msg.clone());
        assert!(load_withdraw_action(&self.deps.storage).unwrap().is_none());
        return response;
    }

    pub fn user_send_compound(&mut self, nasset_profit: Uint256) -> StdResult<Response<Empty>> {
        //this number means nothing
        //because we manually set nasset_profit
        let psi_claimed = Uint256::from(256_000_000u128);

        let env = mock_env();
        let info = mock_info(&"addr9999".to_string(), &vec![]);
        let response =
            crate::contract::execute(self.deps.as_mut(), env, info, ExecuteMsg::Compound {})
                .unwrap();

        assert_eq!(
            response.messages,
            vec![SubMsg::reply_on_success(
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_REWARDS_ADDR.to_string(),
                    msg: to_binary(&NAssetTokenRewardsExecuteMsg::Anyone {
                        anyone_msg: NAssetTokenRewardsAnyoneMsg::ClaimRewards { recipient: None },
                    })
                    .unwrap(),
                    funds: vec![],
                }),
                SubmsgIds::PsiClaimed.id(),
            )]
        );

        // PSI SOLD REPLY
        self.set_psi_balance(psi_claimed);
        let reply_msg = Reply {
            id: SubmsgIds::PsiClaimed.id(),
            result: cosmwasm_std::ContractResult::Ok(SubMsgExecutionResponse {
                events: vec![],
                data: None,
            }),
        };

        let res =
            crate::contract::reply(self.deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();
        assert_eq!(
            res.messages,
            vec![SubMsg::reply_on_success(
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: PSI_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        amount: psi_claimed.into(),
                        contract: PSI_TO_NASSET_PAIR_ADDR.to_string(),
                        msg: to_binary(&AstroportCw20HookMsg::Swap {
                            belief_price: None,
                            max_spread: None,
                            to: None,
                        })
                        .unwrap(),
                    })
                    .unwrap(),
                    funds: vec![],
                },),
                SubmsgIds::PsiSold.id()
            )]
        );

        // PSI SWAPPED
        self.set_psi_balance(Uint256::zero());
        self.increase_nasset_balance(nasset_profit);

        let reply_msg = Reply {
            id: SubmsgIds::PsiSold.id(),
            result: cosmwasm_std::ContractResult::Ok(SubMsgExecutionResponse {
                events: vec![],
                data: None,
            }),
        };
        let response = crate::contract::reply(self.deps.as_mut(), mock_env(), reply_msg.clone());
        assert!(load_withdraw_action(&self.deps.storage).unwrap().is_none());
        return response;
    }

    pub fn set_auto_nasset_supply(&mut self, value: Uint256) {
        self.auto_nasset_supply = value.into();
        self.set_token_supplies();
    }

    pub fn set_nasset_balance(&mut self, value: Uint256) {
        self.nasset_balance = value.into();
        self.set_token_balances();
    }

    pub fn increase_nasset_balance(&mut self, value: Uint256) {
        self.nasset_balance = (Uint256::from(self.nasset_balance) + value).into();
        self.set_token_balances();
    }

    pub fn set_psi_balance(&mut self, value: Uint256) {
        self.psi_balance = value.into();
        self.set_token_balances();
    }

    fn set_token_supplies(&mut self) {
        let supplies = vec![(AUTO_NASSET_TOKEN_ADDR.to_string(), self.auto_nasset_supply)];
        let supplies = HashMap::from_iter(supplies.into_iter());
        self.deps.querier.with_token_supplies(supplies)
    }

    fn set_token_balances(&mut self) {
        self.deps.querier.with_token_balances(&[
            (
                &NASSET_TOKEN_ADDR.to_string(),
                &[(&MOCK_CONTRACT_ADDR.to_string(), &self.nasset_balance)],
            ),
            (
                &PSI_TOKEN_ADDR.to_string(),
                &[(&MOCK_CONTRACT_ADDR.to_string(), &self.psi_balance)],
            ),
        ]);
    }
}
