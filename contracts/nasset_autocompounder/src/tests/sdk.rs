use crate::{
    msg::InstantiateMsg,
    reply_response::MsgInstantiateContractResponse,
    state::{load_config, Config},
    SubmsgIds,
};

use super::{mock_dependencies, WasmMockQuerier};
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::{Api, OwnedDeps, Querier, Reply, Storage, SubMsgExecutionResponse};
use protobuf::Message;

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

        Sdk { deps }
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
}
