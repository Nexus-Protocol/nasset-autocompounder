use crate::{
    msg::InstantiateMsg,
    reply_response::MsgInstantiateContractResponse,
    state::{load_config, Config},
    SubmsgIds,
};

use super::*;
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{
    testing::{mock_env, mock_info},
    Reply, SubMsgExecutionResponse,
};
use protobuf::Message;
use sdk::Sdk;

#[test]
fn proper_initialization() {
    let sdk = Sdk::init();
}
