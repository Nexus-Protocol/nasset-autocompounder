use crate::{
    commands, concat,
    msg::{Cw20HookMsg, NAssetTokenRewardsAnyoneMsg, NAssetTokenRewardsExecuteMsg},
    state::{
        load_config, load_gov_update, load_withdraw_action, remove_gov_update,
        remove_withdraw_action, store_config, store_gov_update, store_withdraw_action, Config,
        GovernanceUpdateState, WithdrawAction,
    },
    SubmsgIds,
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, BlockInfo, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    QuerierWrapper, QueryRequest, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
    WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw20_base::state::TokenInfo;

pub fn update_config(
    deps: DepsMut,
    mut current_config: Config,
    psi_token_addr: Option<String>,
    psi_to_nasset_pair_addr: Option<String>,
    nasset_token_rewards_addr: Option<String>,
) -> StdResult<Response> {
    if let Some(ref psi_token_addr) = psi_token_addr {
        current_config.psi_token = deps.api.addr_validate(psi_token_addr)?;
    }

    if let Some(ref psi_to_nasset_pair_addr) = psi_to_nasset_pair_addr {
        current_config.psi_to_nasset_pair = deps.api.addr_validate(psi_to_nasset_pair_addr)?;
    }

    if let Some(ref nasset_token_rewards_addr) = nasset_token_rewards_addr {
        current_config.nasset_token_rewards = deps.api.addr_validate(nasset_token_rewards_addr)?;
    }

    store_config(deps.storage, &current_config)?;
    Ok(Response::default())
}

pub fn update_governance_addr(
    deps: DepsMut,
    env: Env,
    gov_addr: String,
    seconds_to_wait_for_accept_gov_tx: u64,
) -> StdResult<Response> {
    let current_time = get_time(&env.block);
    let gov_update = GovernanceUpdateState {
        new_governance_contract_addr: deps.api.addr_validate(&gov_addr)?,
        wait_approve_until: current_time + seconds_to_wait_for_accept_gov_tx,
    };
    store_gov_update(deps.storage, &gov_update)?;
    Ok(Response::default())
}

pub fn accept_governance(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let gov_update = load_gov_update(deps.storage)?;
    let current_time = get_time(&env.block);

    if gov_update.wait_approve_until < current_time {
        return Err(StdError::generic_err(
            "too late to accept governance owning",
        ));
    }

    if info.sender != gov_update.new_governance_contract_addr {
        return Err(StdError::generic_err("unauthorized"));
    }

    let new_gov_add_str = gov_update.new_governance_contract_addr.to_string();

    let mut config = load_config(deps.storage)?;
    config.governance_contract = gov_update.new_governance_contract_addr;
    store_config(deps.storage, &config)?;
    remove_gov_update(deps.storage);

    Ok(Response::default().add_attributes(vec![
        ("action", "change_governance_contract"),
        ("new_address", &new_gov_add_str),
    ]))
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Deposit {} => commands::receive_cw20_deposit(deps, env, info, cw20_msg),
        Cw20HookMsg::Withdraw {} => commands::receive_cw20_withdraw(deps, env, info, cw20_msg),
    }
}

pub fn receive_cw20_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let nasset_addr = info.sender;
    // only nAsset contract can execute this message
    let config: Config = load_config(deps.storage)?;
    if nasset_addr != config.nasset_token {
        return Err(StdError::generic_err("unauthorized"));
    }

    //we trust cw20 contract
    let farmer_addr: Addr = Addr::unchecked(cw20_msg.sender);

    deposit_nasset(deps, env, config, farmer_addr, cw20_msg.amount.into())
}

pub fn deposit_nasset(
    deps: DepsMut,
    env: Env,
    config: Config,
    farmer: Addr,
    deposit_amount: Uint256,
) -> StdResult<Response> {
    let auto_nasset_supply: Uint256 =
        query_supply(&deps.querier, &config.auto_nasset_token)?.into();

    let nasset_balance: Uint256 =
        query_token_balance(deps.as_ref(), &config.nasset_token, &env.contract.address).into();

    let is_first_depositor = auto_nasset_supply.is_zero();

    // anAsset tokens to mint:
    // user_share = (deposited_nasset / total_nasset)
    // anAsset_to_mint = anAsset_supply * user_share / (1 - user_share)
    let auto_nasset_to_mint = if is_first_depositor {
        deposit_amount
    } else {
        // 'nasset_supply' can't be zero here, cause we already mint some for first farmer
        auto_nasset_supply * deposit_amount
            / Decimal256::from_uint256(nasset_balance - deposit_amount)
    };

    //0. mint auto_nasset
    Ok(Response::new()
        .add_message(WasmMsg::Execute {
            contract_addr: config.auto_nasset_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: farmer.to_string(),
                amount: auto_nasset_to_mint.into(),
            })?,
            funds: vec![],
        })
        .add_attributes(vec![
            ("action", "deposit_nasset"),
            ("farmer", &farmer.to_string()),
            ("amount", &deposit_amount.to_string()),
        ]))
}

pub fn receive_cw20_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let contract_addr = info.sender;
    // only anAsset contract can execute this message
    let config: Config = load_config(deps.storage)?;
    if contract_addr != config.auto_nasset_token {
        return Err(StdError::generic_err("unauthorized"));
    }

    //we trust cw20 contract
    let farmer_addr: Addr = Addr::unchecked(cw20_msg.sender);

    withdraw_nasset(deps, env, config, farmer_addr, cw20_msg.amount)
}

pub fn withdraw_nasset(
    deps: DepsMut,
    _env: Env,
    config: Config,
    farmer: Addr,
    auto_nasset_to_withdraw_amount: Uint128,
) -> StdResult<Response> {
    //auto_nasset_to_withdraw_amount is not zero here, cw20 contract check it
    store_withdraw_action(
        deps.storage,
        WithdrawAction {
            farmer,
            auto_nasset_amount: auto_nasset_to_withdraw_amount,
        },
    )?;

    Ok(Response::new()
        .add_submessage(SubMsg::reply_always(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.nasset_token_rewards.to_string(),
                msg: to_binary(&NAssetTokenRewardsExecuteMsg::Anyone {
                    anyone_msg: NAssetTokenRewardsAnyoneMsg::ClaimRewards { recipient: None },
                })?,
                funds: vec![],
            }),
            SubmsgIds::PsiClaimed.id(),
        ))
        .add_attributes(vec![("action", "claim_psi")]))
}

pub fn compound(deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response> {
    let config: Config = load_config(deps.storage)?;

    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_success(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.nasset_token_rewards.to_string(),
                msg: to_binary(&NAssetTokenRewardsExecuteMsg::Anyone {
                    anyone_msg: NAssetTokenRewardsAnyoneMsg::ClaimRewards { recipient: None },
                })?,
                funds: vec![],
            }),
            SubmsgIds::PsiClaimed.id(),
        ))
        .add_attributes(vec![("action", "claim_psi")]))
}

pub fn execute_withdraw(deps: DepsMut, env: Env) -> StdResult<Response> {
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
            commands::query_supply(&deps.querier, &config.auto_nasset_token)?.into();

        let nasset_to_withdraw: Uint256 = nasset_balance
            * Uint256::from(withdraw_action.auto_nasset_amount)
            / Decimal256::from_uint256(auto_nasset_supply);

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
                contract_addr: config.auto_nasset_token.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: withdraw_action.auto_nasset_amount,
                })?,
                funds: vec![],
            }))
            .add_attributes(vec![
                ("action", "withdraw"),
                (
                    "auto_nasset_amount_burned",
                    &withdraw_action.auto_nasset_amount.to_string(),
                ),
                ("nasset_amount_withdrawed", &nasset_to_withdraw.to_string()),
            ]))
    } else {
        Ok(Response::new())
    }
}

fn get_time(block: &BlockInfo) -> u64 {
    block.time.seconds()
}

// ====================================================================================

pub fn query_supply(querier: &QuerierWrapper, contract_addr: &Addr) -> StdResult<Uint128> {
    if let Ok(supply) = query_supply_legacy(querier, contract_addr) {
        return Ok(supply);
    }

    query_supply_new(querier, contract_addr)
}

fn query_supply_new(querier: &QuerierWrapper, contract_addr: &Addr) -> StdResult<Uint128> {
    let token_info: TokenInfo = querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: contract_addr.to_string(),
        key: Binary::from(b"token_info"),
    }))?;

    Ok(token_info.total_supply)
}

fn query_supply_legacy(querier: &QuerierWrapper, contract_addr: &Addr) -> StdResult<Uint128> {
    let token_info: TokenInfo = querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: contract_addr.to_string(),
        key: Binary::from(to_length_prefixed(b"token_info")),
    }))?;

    Ok(token_info.total_supply)
}

// ====================================================================================

pub fn query_token_balance(deps: Deps, contract_addr: &Addr, account_addr: &Addr) -> Uint128 {
    if let Ok(balance) = query_token_balance_legacy(&deps, contract_addr, account_addr) {
        return balance;
    }

    if let Ok(balance) = query_token_balance_new(&deps, contract_addr, account_addr) {
        return balance;
    }

    Uint128::zero()
}

fn query_token_balance_new(
    deps: &Deps,
    contract_addr: &Addr,
    account_addr: &Addr,
) -> StdResult<Uint128> {
    // load balance form the cw20 token contract version 0.6+
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: contract_addr.to_string(),
        key: Binary::from(concat(
            &to_length_prefixed(b"balance"),
            account_addr.as_bytes(),
        )),
    }))
}

fn query_token_balance_legacy(
    deps: &Deps,
    contract_addr: &Addr,
    account_addr: &Addr,
) -> StdResult<Uint128> {
    // load balance form the cw20 token contract version 0.2.x
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: contract_addr.to_string(),
        key: Binary::from(concat(
            &to_length_prefixed(b"balance"),
            (deps.api.addr_canonicalize(account_addr.as_str())?).as_slice(),
        )),
    }))
}
