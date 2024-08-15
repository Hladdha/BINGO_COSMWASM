use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128, Uint64, Timestamp};

#[derive(Serialize, Deserialize)]
pub enum ExecuteMsg{
    CreateGame {fee:Uint128, T1:Timestamp, T2:Uint64},
    JoinGame {game_id: u32},
    DrawNumber{game_id: u32},
    GetBoard{game_id: u32},

}
pub enum QueryMsg {
    GetDenom{},
}

pub enum ExecuteMsg{
    CreateGame {},
    JoinGame {game_id: u32},
    DrawNumber{game_id: u32},
    WithdrawWinnings{}
}