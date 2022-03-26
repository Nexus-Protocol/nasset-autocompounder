use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
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
//TODO
pub struct ConfigResponse {
    pub governance_contract: String,
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

// TODO: remove?
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum TerraswapRouterExecuteMsg {
//     /// Execute multiple BuyOperation
//     ExecuteSwapOperations {
//         operations: Vec<TerraswapRouterSwapOperation>,
//         minimum_receive: Option<Uint128>,
//         to: Option<String>,
//     },
// }

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum TerraswapRouterSwapOperation {
//     NativeSwap {
//         offer_denom: String,
//         ask_denom: String,
//     },
//     TerraSwap {
//         offer_asset_info: AssetInfo,
//         ask_asset_info: AssetInfo,
//     },
// }

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum AstroportExecuteMsg {
//     Receive(Cw20ReceiveMsg),
//     /// ProvideLiquidity a user provides pool liquidity
//     ProvideLiquidity {
//         assets: [Asset; 2],
//         slippage_tolerance: Option<Decimal>,
//         auto_stake: Option<bool>,
//         receiver: Option<String>,
//     },
//     /// Swap an offer asset to the other
//     Swap {
//         offer_asset: Asset,
//         belief_price: Option<Decimal>,
//         max_spread: Option<Decimal>,
//         to: Option<String>,
//     },
// }

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

// TODO: remove?
////copypasted from terraswap
//#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
//pub struct Asset {
//    pub info: AssetInfo,
//    pub amount: Uint128,
//}

////copypasted from terraswap
//#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
//#[serde(rename_all = "snake_case")]
//pub enum AssetInfo {
//    Token { contract_addr: Addr },
//    NativeToken { denom: String },
//}
