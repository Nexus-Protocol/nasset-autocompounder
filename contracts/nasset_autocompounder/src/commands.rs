use crate::{
    commands, concat,
    msg::{
        Cw20HookMsg, SpecNassetFarmCw20Msg, SpecNassetFarmMsg, SpecNassetFarmQueryMsg,
        SpecNassetFarmRewardInfoResponse,
    },
    state::{
        load_config, load_gov_update, remove_gov_update, store_config, store_gov_update, Config,
        GovernanceUpdateState,
    },
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, BlockInfo, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, QuerierWrapper, QueryRequest, Response, StdError, StdResult, Uint128, WasmMsg,
    WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw20_base::state::TokenInfo;

pub fn update_config(
    deps: DepsMut,
    mut current_config: Config,
    psi_distributor_addr: Option<String>,
    anchor_overseer_contract_addr: Option<String>,
    anchor_market_contract_addr: Option<String>,
    anchor_custody_basset_contract_addr: Option<String>,
    anc_stable_swap_contract_addr: Option<String>,
    psi_stable_swap_contract_addr: Option<String>,
    basset_vault_strategy_contract_addr: Option<String>,
    claiming_rewards_delay: Option<u64>,
    over_loan_balance_value: Option<Decimal256>,
) -> StdResult<Response> {
    //TODO
    Ok(Response::new())
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
        query_supply(&deps.querier, &config.auto_nasset_token.clone())?.into();

    let nasset_in_autocompounder = get_nasset_in_autocompounder(
        deps.as_ref(),
        &config.nasset_token,
        &config.spec_nasset_farm,
        &env.contract.address,
    )?;

    if nasset_in_autocompounder.is_zero() && !auto_nasset_supply.is_zero() {
        //read comments in 'withdraw_nasset' function for a reason to return error here
        return Err(StdError::generic_err(
            "nAsset balance is zero, but anAsset supply is not! Freeze contract.",
        ));
    }

    // nasset balance in cw20 contract
    // it should be equal to 'deposit_amout',
    // unless someone directly transfer cw20 tokens to this contract without calling 'Deposit'
    let nasset_in_contract_address =
        query_token_balance(deps.as_ref(), &config.nasset_token, &env.contract.address);

    let nasset_balance: Uint256 = nasset_in_autocompounder + nasset_in_contract_address.into();
    let is_first_depositor = deposit_amount == nasset_balance;

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

    //0. send nasset to autocompounder
    //1. mint auto_nasset
    Ok(Response::new()
        .add_messages(vec![
            WasmMsg::Execute {
                contract_addr: config.nasset_token.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: config.spec_nasset_farm.to_string(),
                    amount: nasset_in_contract_address.into(),
                    msg: to_binary(&SpecNassetFarmCw20Msg::bond {
                        staker_addr: None,
                        compound_rate: Some(Decimal::one()),
                    })?,
                })?,
                funds: vec![],
            },
            WasmMsg::Execute {
                contract_addr: config.auto_nasset_token.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: farmer.to_string(),
                    amount: auto_nasset_to_mint.into(),
                })?,
                funds: vec![],
            },
        ])
        .add_attributes(vec![
            ("action", "deposit_basset"),
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

    withdraw_nasset(deps, env, config, farmer_addr, cw20_msg.amount.into())
}

pub fn withdraw_nasset(
    deps: DepsMut,
    env: Env,
    config: Config,
    farmer: Addr,
    auto_nasset_to_withdraw_amount: Uint256,
) -> StdResult<Response> {
    //auto_nasset_to_withdraw_amount is not zero here, cw20 contract check it

    //nasset_in_contract_address is always zero (except Deposit stage)
    let nasset_in_autocompounder = get_nasset_in_autocompounder(
        deps.as_ref(),
        &config.nasset_token,
        &config.spec_nasset_farm,
        &env.contract.address,
    )?;

    let auto_nasset_supply: Uint256 =
        query_supply(&deps.querier, &config.auto_nasset_token.clone())?.into();

    if nasset_in_autocompounder.is_zero() {
        //interesting case - user owns some anAsset, but nAsset balance is zero
        //what we can do here:
        //1. Burn his anAsset, cause they do not have value in that context
        //2. return error. In that case if someone will deposit nAsset those anAsset owners will
        //   own share of his tokens. But I prevent deposists in that case, so contract is kinds "frozen" -
        //   no withdraw and deposits available when nAsset balance is zero. Looks like the best
        //   solution.
        //3. Burn all anAsset supply (not possible with cw20 messages)
        //
        //Second choice is best one in my opinion.
        return Err(StdError::generic_err(
            "nAsset balance is zero, but anAsset supply is not! Freeze contract.",
        ));
    }

    let nasset_to_withdraw: Uint256 = nasset_in_autocompounder * auto_nasset_to_withdraw_amount
        / Decimal256::from_uint256(Uint256::from(auto_nasset_supply));

    //TODO: claim SPEC rewards before withdrawing
    //do it with Reply logic

    //0. withdraw nasset from autocompounder
    //1. send nasset to farmer
    //2. burn anasset
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.spec_nasset_farm.to_string(),
            msg: to_binary(&SpecNassetFarmMsg::unbond {
                asset_token: config.nasset_token.to_string(),
                amount: nasset_to_withdraw.into(),
            })?,
            funds: vec![],
        }))
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.nasset_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: farmer.to_string(),
                amount: nasset_to_withdraw.into(),
            })?,
            funds: vec![],
        }))
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.nasset_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Burn {
                amount: auto_nasset_to_withdraw_amount.into(),
            })?,
            funds: vec![],
        }))
        .add_attributes(vec![
            ("action", "withdraw"),
            ("nasset_amount", &auto_nasset_to_withdraw_amount.to_string()),
        ]))
}

pub fn get_nasset_in_autocompounder(
    deps: Deps,
    nasset_token: &Addr,
    spec_nasset_farm: &Addr,
    account_addr: &Addr,
) -> StdResult<Uint256> {
    let reward_infos: SpecNassetFarmRewardInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: spec_nasset_farm.to_string(),
            msg: to_binary(&SpecNassetFarmQueryMsg::reward_info {
                staker_addr: account_addr.to_string(),
            })?,
        }))?;

    let rewards: Vec<Uint256> = reward_infos
        .reward_infos
        .into_iter()
        .filter(|reward| reward.asset_token == nasset_token.to_string())
        .map(|reward| reward.bond_amount.into())
        .collect();

    Ok(*rewards.first().unwrap_or(&Uint256::zero()))
}

fn get_time(block: &BlockInfo) -> u64 {
    block.time.seconds()
}

// ====================================================================================

pub fn query_supply(querier: &QuerierWrapper, contract_addr: &Addr) -> StdResult<Uint128> {
    if let Ok(supply) = query_supply_legacy(querier, contract_addr) {
        return Ok(supply);
    }

    return query_supply_new(querier, contract_addr);
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

    return Uint128::zero();
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
