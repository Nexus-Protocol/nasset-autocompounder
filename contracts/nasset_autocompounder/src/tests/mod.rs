mod change_config;
mod change_governance_addr;
mod compound;
mod deposit;
mod instantiate;
mod sdk;
mod withdraw;

use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_slice, to_binary, Addr, Coin, ContractResult, OwnedDeps, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use std::collections::HashMap;
use std::hash::Hash;
use terra_cosmwasm::TerraQueryWrapper;

use cw20::TokenInfoResponse;

pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    token_querier: TokenQuerier,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Raw { contract_addr, key }) => {
                let key: &[u8] = key.as_slice();

                let prefix_token_info = b"token_info";
                let prefix_token_info_legacy = to_length_prefixed(b"token_info");
                let prefix_balance = to_length_prefixed(b"balance");

                if key.to_vec() == prefix_token_info || key.to_vec() == prefix_token_info_legacy {
                    let token_supply = match self.token_querier.supplies.get(contract_addr) {
                        Some(supply) => supply,
                        None => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: format!(
                                    "No supply info exists for the contract {}",
                                    contract_addr
                                ),
                                request: key.into(),
                            })
                        }
                    };

                    SystemResult::Ok(ContractResult::from(to_binary(&TokenInfoResponse {
                        name: "some_token_name".to_string(),
                        symbol: "some_token_symbol".to_string(),
                        decimals: 6,
                        total_supply: *token_supply,
                    })))
                } else if key[..prefix_balance.len()].to_vec() == prefix_balance {
                    let key_address: &[u8] = &key[prefix_balance.len()..];
                    let address: Addr = Addr::unchecked(std::str::from_utf8(key_address).unwrap());

                    let balances: &HashMap<String, Uint128> =
                        match self.token_querier.balances.get(contract_addr) {
                            Some(balances) => balances,
                            None => {
                                return SystemResult::Err(SystemError::InvalidRequest {
                                    error: format!(
                                        "No balance info exists for the contract {}",
                                        contract_addr
                                    ),
                                    request: key.into(),
                                })
                            }
                        };

                    let balance = match balances.get(&address.to_string()) {
                        Some(v) => v,
                        None => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: "Balance not found".to_string(),
                                request: key.into(),
                            })
                        }
                    };

                    SystemResult::Ok(ContractResult::from(to_binary(&balance)))
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }

            _ => self.base.handle_query(request),
        }
    }

    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier.balances = array_to_hashmap(balances);
    }

    pub fn with_token_supplies(&mut self, supplies: HashMap<String, Uint128>) {
        self.token_querier.supplies = supplies;
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            token_querier: TokenQuerier::default(),
        }
    }
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    balances: HashMap<String, HashMap<String, Uint128>>,
    supplies: HashMap<String, Uint128>,
}

pub(crate) fn array_to_hashmap<K, V>(
    balances: &[(&String, &[(&K, &V)])],
) -> HashMap<String, HashMap<K, V>>
where
    V: Clone,
    K: Clone + Eq + Hash,
{
    let mut result_map: HashMap<String, HashMap<K, V>> = HashMap::new();
    for (contract_addr, map_values) in balances.iter() {
        let mut contract_balances_map: HashMap<K, V> = HashMap::new();
        for (key, value) in map_values.iter() {
            contract_balances_map.insert((**key).clone(), (**value).clone());
        }

        result_map.insert(contract_addr.to_string(), contract_balances_map);
    }
    result_map
}
