use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Decimal, Uint128};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub nasset_token_addr: String,
    pub spec_nasset_farm_addr: String,
    pub governance_contract_addr: String,
    pub cw20_token_code_id: u64,
    pub collateral_token_symbol: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Governance { governance_msg: GovernanceMsg },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceMsg {
    UpdateConfig {
        psi_distributor_addr: Option<String>,
        anchor_overseer_contract_addr: Option<String>,
        anchor_market_contract_addr: Option<String>,
        anchor_custody_basset_contract_addr: Option<String>,
        anc_stable_swap_contract_addr: Option<String>,
        psi_stable_swap_contract_addr: Option<String>,
        basset_vault_strategy_contract_addr: Option<String>,
        claiming_rewards_delay: Option<u64>,
        over_loan_balance_value: Option<Decimal256>,
    },
    UpdateGovernanceContract {
        gov_addr: String,
        //how long to wait for 'AcceptGovernance' transaction
        seconds_to_wait_for_accept_gov_tx: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Deposit {},
    Withdraw {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub governance_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

// ====================================================================================

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[allow(non_camel_case_types)]
pub enum SpecNassetFarmMsg {
    unbond {
        asset_token: String,
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[allow(non_camel_case_types)]
pub enum SpecNassetFarmQueryMsg {
    reward_info { staker_addr: String },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SpecNassetFarmRewardInfoResponse {
    pub staker_addr: String,
    pub reward_infos: Vec<SpecNassetFarmRewardInfoResponseItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SpecNassetFarmRewardInfoResponseItem {
    pub asset_token: String,
    pub farm_share_index: Decimal,
    pub auto_spec_share_index: Decimal,
    pub stake_spec_share_index: Decimal,
    pub bond_amount: Uint128,
    pub auto_bond_amount: Uint128,
    pub stake_bond_amount: Uint128,
    pub farm_share: Uint128,
    pub spec_share: Uint128,
    pub auto_bond_share: Uint128,
    pub stake_bond_share: Uint128,
    pub pending_farm_reward: Uint128,
    pub pending_spec_reward: Uint128,
    pub deposit_amount: Option<Uint128>,
    pub deposit_time: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[allow(non_camel_case_types)]
pub enum SpecNassetFarmCw20Msg {
    bond {
        staker_addr: Option<String>,
        compound_rate: Option<Decimal>,
    },
}
