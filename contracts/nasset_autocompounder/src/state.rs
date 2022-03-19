use cw_storage_plus::Item;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, StdResult, Storage};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub nasset_token: Addr,
    pub auto_nasset_token: Addr,
    pub spec_nasset_farm: Addr,
    pub governance_contract: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GovernanceUpdateState {
    pub new_governance_contract_addr: Addr,
    pub wait_approve_until: u64,
}

static KEY_CONFIG: Item<Config> = Item::new("config");

static KEY_GOVERNANCE_UPDATE: Item<GovernanceUpdateState> = Item::new("gov_update");

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    KEY_CONFIG.load(storage)
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    KEY_CONFIG.save(storage, config)
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

pub fn load_gov_update(storage: &dyn Storage) -> StdResult<GovernanceUpdateState> {
    KEY_GOVERNANCE_UPDATE.load(storage)
}

pub fn store_gov_update(
    storage: &mut dyn Storage,
    gov_update: &GovernanceUpdateState,
) -> StdResult<()> {
    KEY_GOVERNANCE_UPDATE.save(storage, gov_update)
}

pub fn remove_gov_update(storage: &mut dyn Storage) -> () {
    KEY_GOVERNANCE_UPDATE.remove(storage)
}
