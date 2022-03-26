use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};

use crate::msg::{
    AstroportCw20HookMsg, AutoNassetValueResponse, ConfigResponse, ExecuteMsg, GovernanceMsg,
    InstantiateMsg, MigrateMsg, QueryMsg,
};
use crate::reply_response::MsgInstantiateContractResponse;
use crate::state::Config;
use crate::{
    commands,
    state::{
        load_config, load_withdraw_action, remove_withdraw_action, set_auto_nasset_token_addr,
        store_config,
    },
    SubmsgIds,
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::Cw20ExecuteMsg;
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
        auto_nasset_token: Addr::unchecked(""),
        psi_token: deps.api.addr_validate(&msg.psi_token_addr)?,
        psi_to_nasset_pair: deps.api.addr_validate(&msg.psi_to_nasset_pair_addr)?,
        governance_contract: deps.api.addr_validate(&msg.governance_contract_addr)?,
        nasset_token_rewards: deps.api.addr_validate(&msg.nasset_token_rewards_addr)?,
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
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
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

        SubmsgIds::PsiClaimed => {
            let config = load_config(deps.storage)?;
            let psi_balance = commands::query_token_balance(
                deps.as_ref(),
                &config.psi_token,
                &env.contract.address,
            );

            Ok(Response::new().add_submessage(SubMsg::reply_on_success(
                WasmMsg::Execute {
                    contract_addr: config.psi_token.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        amount: psi_balance,
                        contract: config.psi_to_nasset_pair.to_string(),
                        msg: to_binary(&AstroportCw20HookMsg::Swap {
                            belief_price: None,
                            max_spread: None,
                            to: None,
                        })?,
                    })?,
                    funds: vec![],
                },
                SubmsgIds::PsiSold.id(),
            )))
        }

        SubmsgIds::PsiSold => {
            let config = load_config(deps.storage)?;
            if let Some(withdraw_action) = load_withdraw_action(deps.storage)? {
                remove_withdraw_action(deps.storage)?;

                let nasset_balance: Uint256 = commands::query_token_balance(
                    deps.as_ref(),
                    &config.nasset_token,
                    &env.contract.address,
                )
                .into();

                let auto_nasset_supply: Uint256 =
                    commands::query_supply(&deps.querier, &config.auto_nasset_token.clone())?
                        .into();

                let nasset_to_withdraw: Uint256 = nasset_balance
                    * Uint256::from(withdraw_action.auto_nasset_amount)
                    / Decimal256::from_uint256(Uint256::from(auto_nasset_supply));

                //0. send nasset to farmer
                //1. burn anasset
                Ok(Response::new()
                    .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.nasset_token.to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Transfer {
                            recipient: withdraw_action.farmer.to_string(),
                            amount: nasset_to_withdraw.into(),
                        })?,
                        funds: vec![],
                    }))
                    .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.nasset_token.to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Burn {
                            amount: withdraw_action.auto_nasset_amount,
                        })?,
                        funds: vec![],
                    }))
                    .add_attributes(vec![
                        ("action", "withdraw"),
                        ("auto_nasset_amount_burned", &nasset_to_withdraw.to_string()),
                        (
                            "nasset_amount_withdrawed",
                            &withdraw_action.auto_nasset_amount.to_string(),
                        ),
                    ]))
            } else {
                Ok(Response::new())
            }
        }
    }
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),
        ExecuteMsg::Compound {} => commands::compound(deps, env, info),

        ExecuteMsg::AcceptGovernance {} => commands::accept_governance(deps, env, info),

        ExecuteMsg::Governance { governance_msg } => {
            let config: Config = load_config(deps.storage)?;
            if info.sender != config.governance_contract {
                return Err(StdError::generic_err("unauthorized"));
            }

            match governance_msg {
                GovernanceMsg::UpdateConfig {
                    psi_token_addr,
                    psi_to_nasset_pair_addr,
                    nasset_token_rewards_addr,
                } => commands::update_config(
                    deps,
                    config,
                    psi_token_addr,
                    psi_to_nasset_pair_addr,
                    nasset_token_rewards_addr,
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
        QueryMsg::AutoNassetValue { amount } => {
            to_binary(&query_auto_nasset_value(deps, env, amount)?)
        }
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;
    Ok(ConfigResponse {
        nasset_token_addr: config.nasset_token.to_string(),
        auto_nasset_token_addr: config.auto_nasset_token.to_string(),
        psi_token_addr: config.psi_token.to_string(),
        psi_to_nasset_pair_addr: config.psi_to_nasset_pair.to_string(),
        governance_contract_addr: config.governance_contract.to_string(),
        nasset_token_rewards_addr: config.nasset_token_rewards.to_string(),
    })
}

pub fn query_auto_nasset_value(
    deps: Deps,
    env: Env,
    amount: Uint128,
) -> StdResult<AutoNassetValueResponse> {
    let config: Config = load_config(deps.storage)?;

    let nasset_balance: Uint256 =
        commands::query_token_balance(deps, &config.nasset_token, &env.contract.address).into();

    let auto_nasset_supply: Uint256 =
        commands::query_supply(&deps.querier, &config.auto_nasset_token.clone())?.into();

    let nasset_amount: Uint256 = nasset_balance * Uint256::from(amount)
        / Decimal256::from_uint256(Uint256::from(auto_nasset_supply));

    Ok(AutoNassetValueResponse {
        nasset_amount: nasset_amount.into(),
    })
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
