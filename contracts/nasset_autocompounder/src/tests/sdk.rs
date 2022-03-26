use crate::{
    msg::{Cw20HookMsg, ExecuteMsg, InstantiateMsg},
    reply_response::MsgInstantiateContractResponse,
    state::{load_config, Config},
    SubmsgIds,
};

use super::{mock_dependencies, WasmMockQuerier};
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    to_binary, Api, Empty, OwnedDeps, Querier, Reply, Response, StdResult, Storage,
    SubMsgExecutionResponse, Uint128,
};
use cw20::Cw20ReceiveMsg;
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
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            nasset_token_addr: NASSET_TOKEN_ADDR.to_string(),
            psi_token_addr: PSI_TOKEN_ADDR.to_string(),
            psi_to_nasset_pair_addr: PSI_TO_NASSET_PAIR_ADDR.to_string(),
            governance_contract_addr: GOVERNANCE_CONTRACT_ADDR.to_string(),
            cw20_token_code_id: CW20_TOKEN_CODE_ID,
            nasset_token_rewards_addr: NASSET_TOKEN_REWARDS_ADDR.to_string(),
            collateral_token_symbol: COLLATERAL_TOKEN_SYMBOL.to_string(),
        };

        let env = mock_env();

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

    pub fn user_withdraw(&mut self, address: &str, amount: Uint128) -> StdResult<Response<Empty>> {
        let cw20_deposit_msg = Cw20ReceiveMsg {
            sender: address.to_string(),
            amount,
            msg: to_binary(&Cw20HookMsg::Withdraw {}).unwrap(),
        };

        let info = mock_info(AUTO_NASSET_TOKEN_ADDR, &vec![]);
        crate::contract::execute(
            self.deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Receive(cw20_deposit_msg),
        )
    }

    pub fn user_send_compound(&mut self) -> StdResult<Response<Empty>> {
        let env = mock_env();
        let info = mock_info(&"addr9999".to_string(), &vec![]);
        crate::contract::execute(self.deps.as_mut(), env, info, ExecuteMsg::Compound {})
    }

    pub fn set_auto_nasset_supply(&mut self, value: Uint256) {
        self.auto_nasset_supply = value.into();
        self.set_token_supplies();
    }

    pub fn set_nasset_balance(&mut self, value: Uint256) {
        self.nasset_balance = value.into();
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
