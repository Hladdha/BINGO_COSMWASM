use cosmwasm_std::{
    Binary, Deps, DepsMut, Response, StdResult, to_binary, Uint128, BankMsg, coins
};
use cosmwasm_std::{Addr, Env, MessageInfo, StdError, Timestamp, Event};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

const BOARD_SIZE: usize = 5;
const COUNT: Item<u32> = Item::new("count");
pub const GAMES: Map<u32, Game> = Map::new("games");
pub const DENOM: Item<String> = Item::new("denom");

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    
    DENOM.save(deps.storage, &msg.donation_denom)?;

    Ok(Response::new())
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub players: Vec<Addr>,
    pub pot: Uint128,
    pub entry_fee: Uint128,
    pub join_duration: Timestamp,
    pub turn_duration: Timestamp,
}


fn create_game(
    deps: &mut DepsMut,
    _env: Env,
    _info: MessageInfo,
    entry_fee: Uint128,
    join_duration: Timestamp,
    turn_duration: Timestamp,
) -> StdResult<Response> {
    let mut count = COUNT.load(deps.storage)?;

    count += 1;
    let game = Game {
        players: Vec::new(),
        pot: Uint128::zero(),
        entry_fee,
        join_duration,
        turn_duration,
    };

    GAMES.save(deps.storage,count,&game);
    COUNT.save(deps.storage, &count)?;
    Ok(Response::default())
}

fn join_game(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    game_id: u32,
) -> Result<Response, StdError> {
    let mut game = GAMES.load(deps.storage, game_id)?;
    let invest: String = DENOM.load(deps.storage)?;
    
    if env.block.time < game.join_duration {
        return Err(StdError::generic_err("Game is not open for joining"));
    }

    if info.funds[0].amount != game.entry_fee {
        return Err(StdError::generic_err("Invalid entry fee"));    
    }

    game.pot += info.funds[0].amount;
    game.players.push(info.sender);
    GAMES.update(deps.storage,game_id, );

    Ok(Response::default().add_event(Event::new("player_join").add_attribute("address", info.sender)))
}



fn draw_number(deps: DepsMut, _env: Env, _info: MessageInfo, game_id: u32) -> Result<Response, StdError> {

    let mut game = GAMES.load(deps.storage, game_id)?;


    let previous_block_number = _env.block.height;
    let previous_block_hash = api.blockhash(&previous_block_number.into());

    Ok(Response::default())
}

fn withdraw_winnings(deps: DepsMut, _env: Env, _info: MessageInfo) -> Result<Response, StdError> {
    let denom: String = DENOM.load(deps.storage)?;

    let messages = BankMsg::Send {
        to_address: _info.sender.to_string(),
        amount: coins(100, &denom)
    };
    
    let resp = Response::new()
    .add_messages(messages);

    Ok(resp)
}