use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

mod contract;
mod state;
mod msg;