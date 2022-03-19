use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdError, StdResult, SubMsg, WasmMsg,
};

use crate::msg::{ConfigResponse, ExecuteMsg, GovernanceMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::reply_response::MsgInstantiateContractResponse;
use crate::state::Config;
use crate::{
    commands,
    state::{load_config, set_auto_nasset_token_addr, store_config},
    SubmsgIds,
};
use cw20::MinterResponse;
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
use protobuf::Message;
use std::convert::TryFrom;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        nasset_token: deps.api.addr_validate(&msg.nasset_token_addr)?,
        spec_nasset_farm: deps.api.addr_validate(&msg.spec_nasset_farm_addr)?,
        governance_contract: deps.api.addr_validate(&msg.governance_contract_addr)?,
        auto_nasset_token: Addr::unchecked(""),
    };
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_submessage(SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: Some(config.governance_contract.to_string()),
            code_id: msg.cw20_token_code_id,
            msg: to_binary(&Cw20InstantiateMsg {
                name: format!(
                    "n{} autocompounder share representation",
                    msg.collateral_token_symbol
                ),
                symbol: format!("aun{}", msg.collateral_token_symbol),
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.to_string(),
                    cap: None,
                }),
                marketing: None,
            })?,
            funds: vec![],
            label: "".to_string(),
        }),
        SubmsgIds::InitANAsset.id(),
    )))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let submessage_enum = SubmsgIds::try_from(msg.id)?;
    match submessage_enum {
        SubmsgIds::InitANAsset => {
            let data = msg.result.unwrap().data.unwrap();
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
                .map_err(|_| {
                    StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
                })?;

            let auto_nasset_token_addr = res.get_contract_address();
            set_auto_nasset_token_addr(deps.storage, Addr::unchecked(auto_nasset_token_addr))?;

            Ok(Response::new().add_attributes(vec![
                ("action", "auto_nasset_token_initialized"),
                ("auto_nasset_token_addr", auto_nasset_token_addr),
            ]))
        }
    }
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),

        ExecuteMsg::Governance { governance_msg } => {
            let config: Config = load_config(deps.storage)?;
            if info.sender != config.governance_contract {
                return Err(StdError::generic_err("unauthorized"));
            }

            match governance_msg {
                //TODO
                GovernanceMsg::UpdateConfig {
                    psi_distributor_addr,
                    anchor_overseer_contract_addr,
                    anchor_market_contract_addr,
                    anchor_custody_basset_contract_addr,
                    anc_stable_swap_contract_addr,
                    psi_stable_swap_contract_addr,
                    basset_vault_strategy_contract_addr,
                    claiming_rewards_delay,
                    over_loan_balance_value,
                } => commands::update_config(
                    deps,
                    config,
                    psi_distributor_addr,
                    anchor_overseer_contract_addr,
                    anchor_market_contract_addr,
                    anchor_custody_basset_contract_addr,
                    anc_stable_swap_contract_addr,
                    psi_stable_swap_contract_addr,
                    basset_vault_strategy_contract_addr,
                    claiming_rewards_delay,
                    over_loan_balance_value,
                ),

                GovernanceMsg::UpdateGovernanceContract {
                    gov_addr,
                    seconds_to_wait_for_accept_gov_tx,
                } => commands::update_governance_addr(
                    deps,
                    env,
                    gov_addr,
                    seconds_to_wait_for_accept_gov_tx,
                ),
            }
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        //TODO
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;
    Ok(ConfigResponse {
        governance_contract: config.governance_contract.to_string(),
    })
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
