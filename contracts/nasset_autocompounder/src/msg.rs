use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, Uint128};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub nasset_token_addr: String,
    pub auto_nasset_token_addr: String,
    pub psi_token_addr: String,
    pub psi_to_nasset_pair_addr: String,
    pub governance_contract_addr: String,
    pub cw20_token_code_id: u64,
    pub nasset_token_rewards_addr: String,
    pub collateral_token_symbol: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Governance { governance_msg: GovernanceMsg },
    AcceptGovernance {},
    Compound {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceMsg {
    UpdateConfig {
        nasset_token_addr: Option<String>,
        auto_nasset_token_addr: Option<String>,
        psi_token_addr: Option<String>,
        psi_to_nasset_pair_addr: Option<String>,
        nasset_token_rewards_addr: Option<String>,
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
    AutoNassetValue { amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub nasset_token_addr: String,
    pub auto_nasset_token_addr: String,
    pub psi_token_addr: String,
    pub psi_to_nasset_pair_addr: String,
    pub governance_contract_addr: String,
    pub nasset_token_rewards_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AutoNassetValueResponse {
    pub nasset_amount: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

// ====================================================================================

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NAssetTokenRewardsAnyoneMsg {
    ClaimRewards { recipient: Option<String> },
    //Claim rewards for some address, rewards will be sent to it, not to sender!
    ClaimRewardsForSomeone { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NAssetTokenRewardsExecuteMsg {
    Anyone {
        anyone_msg: NAssetTokenRewardsAnyoneMsg,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AstroportCw20HookMsg {
    /// Sell a given amount of asset
    Swap {
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        to: Option<String>,
    },
    WithdrawLiquidity {},
}
