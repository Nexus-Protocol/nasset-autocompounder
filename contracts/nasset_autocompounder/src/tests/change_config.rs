use crate::{
    msg::{ExecuteMsg, GovernanceMsg},
    state::load_config,
};

use super::sdk::{Sdk, GOVERNANCE_CONTRACT_ADDR};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::StdError;

#[test]
fn fail_to_change_config_if_sender_is_not_governance() {
    let mut sdk = Sdk::init();

    let change_config_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateConfig {
            psi_token_addr: None,
            psi_to_nasset_pair_addr: None,
            nasset_token_rewards_addr: None,
        },
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    let res = crate::contract::execute(sdk.deps.as_mut(), env, info, change_config_msg);
    assert!(res.is_err());
    if let StdError::GenericErr { msg, .. } = res.err().unwrap() {
        assert_eq!("unauthorized", msg);
    } else {
        panic!("wrong error");
    }
}

#[test]
fn success_to_change_config_if_sender_governance() {
    let mut sdk = Sdk::init();

    let new_psi_token_addr = "addr9992".to_string();
    let new_psi_to_nasset_pair_addr = "addr9991".to_string();
    let new_nasset_token_rewards_addr = "addr9990".to_string();

    let change_config_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateConfig {
            psi_token_addr: Some(new_psi_token_addr.clone()),
            psi_to_nasset_pair_addr: Some(new_psi_to_nasset_pair_addr.clone()),
            nasset_token_rewards_addr: Some(new_nasset_token_rewards_addr.clone()),
        },
    };

    let env = mock_env();
    let info = mock_info(GOVERNANCE_CONTRACT_ADDR, &[]);
    crate::contract::execute(sdk.deps.as_mut(), env, info, change_config_msg).unwrap();

    let config = load_config(&sdk.deps.storage).unwrap();
    assert_eq!(new_psi_token_addr, config.psi_token);
    assert_eq!(new_psi_to_nasset_pair_addr, config.psi_to_nasset_pair);
    assert_eq!(new_nasset_token_rewards_addr, config.nasset_token_rewards);
}
