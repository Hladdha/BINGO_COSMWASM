use cosmwasm_std::Addr;
use serde::{Deserialize, Serialize};


pub enum ExecuteMsg{
    CreateGame {},
    JoinGame {game_id: u32},
    DrawNumber{game_id: u32},
    WithdrawWinnings{}
}