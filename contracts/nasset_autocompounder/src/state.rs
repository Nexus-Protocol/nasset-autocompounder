use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, StdError, StdResult, Storage, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub nasset_token: Addr,
    pub auto_nasset_token: Addr,
    pub psi_token: Addr,
    pub psi_to_nasset_pair: Addr,
    pub governance_contract: Addr,
    pub nasset_token_rewards: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GovernanceUpdateState {
    pub new_governance_contract_addr: Addr,
    pub wait_approve_until: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WithdrawAction {
    pub farmer: Addr,
    pub auto_nasset_amount: Uint128,
}

static KEY_CONFIG: Item<Config> = Item::new("config");
static KEY_WITHDRAW_ACTION: Item<Option<WithdrawAction>> = Item::new("withdraw_action");
static USERS_SHARE: Map<&Addr, Uint128> = Map::new("shares");

static KEY_GOVERNANCE_UPDATE: Item<GovernanceUpdateState> = Item::new("gov_update");

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    KEY_CONFIG.load(storage)
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    KEY_CONFIG.save(storage, config)
}

pub fn load_withdraw_action(storage: &dyn Storage) -> StdResult<Option<WithdrawAction>> {
    KEY_WITHDRAW_ACTION.load(storage)
}

pub fn store_withdraw_action(
    storage: &mut dyn Storage,
    withdraw_action: WithdrawAction,
) -> StdResult<()> {
    KEY_WITHDRAW_ACTION.update(storage, |v| {
        if v.is_some() {
            Err(StdError::generic_err("Repetitive reply definition!"))
        } else {
            Ok(Some(withdraw_action))
        }
    })?;
    Ok(())
}

pub fn remove_withdraw_action(storage: &mut dyn Storage) -> StdResult<()> {
    KEY_WITHDRAW_ACTION.save(storage, &None)
}

pub fn config_set_nasset_token(storage: &mut dyn Storage, nasset_token: Addr) -> StdResult<Config> {
    KEY_CONFIG.update(storage, |mut config: Config| -> StdResult<_> {
        config.nasset_token = nasset_token;
        Ok(config)
    })
}

pub fn set_auto_nasset_token_addr(
    storage: &mut dyn Storage,
    auto_nasset_token: Addr,
) -> StdResult<Config> {
    KEY_CONFIG.update(storage, |mut config: Config| -> StdResult<_> {
        config.auto_nasset_token = auto_nasset_token;
        Ok(config)
    })
}

pub fn may_load_bank(storage: &dyn Storage, addr: &Addr) -> StdResult<Option<Uint128>> {
    USERS_SHARE.may_load(storage, addr)
}

pub fn load_bank(storage: &dyn Storage, addr: &Addr) -> StdResult<Uint128> {
    may_load_bank(storage, addr).map(|res| res.unwrap_or_default())
}

pub fn store_bank(storage: &mut dyn Storage, addr: &Addr, share: &Uint128) -> StdResult<()> {
    USERS_SHARE.save(storage, addr, share)
}

pub fn load_gov_update(storage: &dyn Storage) -> StdResult<GovernanceUpdateState> {
    KEY_GOVERNANCE_UPDATE.load(storage)
}

pub fn store_gov_update(
    storage: &mut dyn Storage,
    gov_update: &GovernanceUpdateState,
) -> StdResult<()> {
    KEY_GOVERNANCE_UPDATE.save(storage, gov_update)
}

pub fn remove_gov_update(storage: &mut dyn Storage) {
    KEY_GOVERNANCE_UPDATE.remove(storage)
}
