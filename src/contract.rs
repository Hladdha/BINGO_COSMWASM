use cosmwasm_std::{Deps, DepsMut, Response, StdResult, Uint128, BankMsg, coins, entry_point};
use cosmwasm_std::{Addr, Env, MessageInfo, StdError, Timestamp, Event};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};
use crate::msg::{ExecuteMsg};
use sha3::{Digest, Keccak256};

const BOARD_SIZE: usize = 5;
const COUNT: Item<u32> = Item::new("count");
pub const GAMES: Map<u32, Game> = Map::new("games");
pub const DENOM: Item<String> = Item::new("denom");
pub const PLAYER_BOARD: Map<(Addr, u32), u32> = Map::new("players");

const PATTERNS: [[u8; 5]; 12] = [
    [0, 1, 2, 3, 4],
    [5, 6, 7, 8, 9],
    [10, 11, 12, 13, 0],
    [14, 15, 16, 17, 18],
    [19, 20, 21, 22, 23],
    [0, 5, 10, 14, 19],
    [1, 6, 11, 15, 20],
    [2, 7, 16, 21, 0],
    [3, 8, 12, 17, 22],
    [4, 9, 13, 18, 23],
    [0, 6, 17, 23, 0],
    [4, 8, 15, 19, 0],
];


#[derive(Serialize, Deserialize)]
pub struct Game {
    pub players: Vec<Addr>,
    pub pot: Uint128,
    pub entry_fee: Uint128,
    pub game_finished: bool,
    pub join_duration: Timestamp,
    pub turn_duration: Timestamp,
}

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    
    DENOM.save(deps.storage, &msg.Denom)?;

    Ok(Response::new())
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
        game_finished: false,
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
    let block_height = env.block.height;

    let mut hasher = Sha3_256::new();

    hasher.update(info.sender.as_bytes());

    hasher.update(game_id.to_be_bytes());

    hasher.update(block_height.to_be_bytes());

    let hash_result = hasher.finalize();

    let empty = PLAYER_BOARD.may_load(&store, (info.sender, game_id))?;
    assert_eq!(None, empty);
    PLAYER_BOARD.save(&mut store, (info.sender, game_id), hash_result);

    if env.block.time < game.join_duration {
        return Err(StdError::generic_err("Game is not open for joining"));
    }

    if game.game_finished == true{
        return Err(StdError::generic_err("Game is already Finished"));
    }

    if info.funds[0].amount != game.entry_fee {
        return Err(StdError::generic_err("Invalid entry fee"));    
    }

    game.pot += info.funds[0].amount;
    game.players.push(info.sender);
    GAMES.update(deps.storage,game_id,|v| {
        Ok(v.unwrap_or_default())
    });

    Ok(Response::default().add_event(Event::new("player_join").add_attribute("address", info.sender)))
}



fn draw_number(deps: DepsMut, _env: Env, _info: MessageInfo, game_id: u32) -> Result<Response, StdError> {

    let mut game = GAMES.load(deps.storage, game_id)?;


    let previous_block_number = _env.block.height;

    Ok(Response::default())
}

fn withdraw_winnings(deps: DepsMut, _env: Env, _info: MessageInfo) -> Result<Response, StdError> {
    let denom: String = DENOM.load(deps.storage)?;
    
    let resp = Response::new()
    .add_message(BankMsg::Send {
        to_address: _info.sender.to_string(),
        amount: coins(100, &denom)
    });

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StdError> {
    match msg {
        ExecuteMsg::CreateGame {} => create_game(deps, env, info),
        ExecuteMsg::JoinGame { game_id } => join_game(deps, env, info, game_id),
        ExecuteMsg::DrawNumber { game_id } => draw_number(deps, env, info, game_id),
        ExecuteMsg::WithdrawWinnings {} => withdraw_winnings(deps, env, info),
    }
}


pub fn get_board(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    game_id: u32
) -> Result<Response, StdError> {
    let board = PlayerBoard {
        player: info.sender,
        game_id: game_id,
    };

    let mut player_board: PlayerBoard = PLAYER_BOARD.load(deps.storage, board)?;

    Ok(Response::default())
}