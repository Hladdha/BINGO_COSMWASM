use cosmwasm_std::{DepsMut, Response, StdResult, Uint128, Uint64, BankMsg, coins, entry_point};
use cosmwasm_std::{Addr, Env, MessageInfo, StdError, Timestamp, Event, Deps};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};
use crate::msg::{ExecuteMsg, QueryMsg};
use sha3::{Digest, Keccak256};
use std::collections::HashMap;
use base64ct::{Base64, Encoding};

const BOARD_SIZE: usize = 5;
const COUNT: Item<u32> = Item::new("count");
pub const GAMES: Map<u32, Game> = Map::new("games");
pub const DENOM: Item<String> = Item::new("denom");
pub const PLAYER_BOARD: Map<(Addr, u32), &[u8]> = Map::new("player");

const PATTERNS: [[usize; 5]; 12] = [
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


#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct Game {
    pub players: Vec<Addr>,
    pub pot: Uint128,
    pub entry_fee: Uint128,
    pub game_finished: bool,
    pub join_duration: Timestamp,
    pub turn_duration: Uint64,
    pub last_draw_time: Timestamp,
    pub numbers: HashMap<u8, bool>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct InstantiateMsg {
    pub Denom: String,
}

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<String> {
    let denom = msg.Denom;
    let count = 0;
    COUNT.save(deps.storage, &count); 
    DENOM.save(deps.storage, &denom);
    Ok(denom)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Response> {
    match _msg{
        QueryMsg::GetDenom {} => {
            get_denom(_deps);
            return Ok(Response::new())
        }
    }
}

pub fn get_denom(_deps: Deps) -> Option<String> {
    let denom = DENOM.load(_deps.storage).unwrap();
    Some(denom)
}

pub fn query_board(_deps: Deps, addr: Addr, game_id : u32) -> Option<&[u8]> {
    let board = PLAYER_BOARD.may_load(_deps.storage, (addr , game_id)).unwrap();
    board
}
 

pub fn create_game(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    entry_fee: Uint128,
    join_duration: Timestamp,
    turn_duration: Uint64,
) -> Option<Game> {
    let mut count = COUNT.load(deps.storage).unwrap();

    count += 1;
    let game = Game {
        players: Vec::new(),
        pot: Uint128::zero(),
        entry_fee,
        game_finished: false,
        join_duration,
        turn_duration,
        last_draw_time: join_duration,
        numbers: HashMap::new()
    };

    GAMES.save(deps.storage,count,&game);
    COUNT.save(deps.storage, &count);
    Some(game)

}

pub fn join_game(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    game_id: u32,
) -> Result<Response, StdError> {
    let mut game = GAMES.load(deps.storage, game_id)?;
    let block_height = env.block.height;
    let address = &info.sender;
    let empty = PLAYER_BOARD.may_load(deps.storage, (address.clone(), game_id)).unwrap();

    if empty.is_none() {
    let mut hasher = Keccak256::new();

    hasher.update(address.as_bytes());
    hasher.update(game_id.to_be_bytes());
    hasher.update(block_height.to_be_bytes());

    let hash_result = hasher.finalize();
    let bytes = hash_result.as_slice();
    let first_24_bytes = &bytes[..24];
    let base64_hash = Base64::encode_string(&hash_result.as_slice());

    PLAYER_BOARD.save(deps.storage, (info.sender.clone(), game_id), &first_24_bytes);

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
    game.players.push(info.sender.clone());
    GAMES.update(deps.storage,game_id,|_games: Option<Game>| -> StdResult<_> { Ok(game) });

    Ok(Response::default().add_event(Event::new("player_join").add_attribute("address", info.sender)))
} else {
    Err(StdError::generic_err("cant join same game twice"))
}
}


pub fn draw_number(deps: DepsMut, _env: Env, _info: MessageInfo, game_id: u32) -> StdResult<u8> {
    let current_time = _env.block.time;
    let mut game = GAMES.load(deps.storage, game_id)?;

    if !(game.game_finished) {
        if current_time.seconds() < game.last_draw_time.plus_seconds(game.turn_duration.into()).seconds() {
            return Err(StdError::generic_err("wait for turn"));
        }
    }else {
        if current_time < game.join_duration {
            return Err(StdError::generic_err("game is not started yet"));
        }}
    let block_height = _env.block.height;
    let mut hasher = Keccak256::new();

    hasher.update(block_height.to_string());
    hasher.update(current_time.to_string());
    let number_drawn = hasher.finalize();
    
    game.numbers.insert(number_drawn[0], true);
    Ok(number_drawn[0])
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
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::CreateGame {fee , T1, T2} => {
            create_game(deps, env, info, fee, T1, T2);
            return Ok(Response::new())},
        ExecuteMsg::JoinGame { game_id } => join_game(deps, env, info, game_id),
        ExecuteMsg::DrawNumber { game_id } => {
            draw_number(deps, env, info, game_id);
            return Ok(Response::new())
        }
        ExecuteMsg::GetBoard { game_id } => {
            get_board(deps, env, info, game_id);
            return Ok(Response::new())
        }
    }
}

pub fn get_board(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    game_id: u32
) -> Result<[u8; 24], StdError> {
    let player_board = PLAYER_BOARD.may_load(deps.storage, (info.sender , game_id))?;
    if player_board.is_none() {
        return Err(StdError::generic_err("game is not started yet"));
    }   
    let binding = player_board.clone().unwrap();
    let bytes  = binding.as_bytes();
    let mut  board:[u8; 24] = [0; 24];
    for n in 0..24 {
        board[n] = bytes[31-n].clone();
    }

    Ok(board)
}

pub fn bingo(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    game_id: u32
) -> Result<Response, StdError>{
    let mut game = GAMES.load(deps.storage, game_id)?;
    let player_board = PLAYER_BOARD.may_load(deps.storage, (info.sender.clone(), game_id))?;
    if player_board.is_none() {
        return Err(StdError::generic_err("you didnt join the game"));
    }
    let mut result = true;
    let patterns = PATTERNS;

    for n in 0..12 {
        let pattern =  patterns[n];
        let patternlength = if n == 2 || n==7 || n==10 || n==11 {
            4
        }else { 5 };

        for i in 0..patternlength {
            result = result & game.numbers[&player_board.clone().unwrap().as_bytes()[31 - pattern[i]]];
        }
        if result { break };
        if n < 11 {result =true;} 
    }
    if !result {
        return Err(StdError::generic_err("you didnt win"));
    }
    game.game_finished = true;
    GAMES.update(deps.storage,game_id,|_games: Option<Game>| -> StdResult<_> { Ok(game) })?;
    withdraw_winnings(deps, env, info);

    Ok(Response::default().add_event(Event::new("game_finished").add_attribute("Game Finished", game_id.to_string())))
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    use super::*;

    fn _do_instansiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Option<String> {
        let denom = msg.Denom;
        let count = 0;
        COUNT.save(deps.storage, &count);
        DENOM.save(deps.storage, &denom);
        Some(denom)
    }

    mod instantiate {
        use super::*;

        #[test]
        fn basic(){
            let mut deps = mock_dependencies();
            let info = mock_info("creator", &[]);
            let env = mock_env();
            let instantiate_msg = InstantiateMsg {
                Denom : String::from("umlg"),
            };
            let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
            assert_eq!(String::from("umlg"), res );
        }
    }
    mod Bingo {
        use cosmwasm_std::Attribute;

        use super::*;

        #[test]
        fn create_game_test(){
            let mut deps =  mock_dependencies();
            let _info = mock_info("creator", &[]);
            let _env = mock_env();
            let env  = mock_env();
            let info = mock_info("creator", &[]);
            let entry_fee = Uint128::new(100);
            let join_duration = env.block.time.plus_seconds(1000);
            let turn_duration = Uint64::new(1000);
            let msg = InstantiateMsg {
                Denom : String::from("umlg"),
            };

            _do_instansiate(deps.as_mut(), env, info, msg);
            
            let game = create_game(deps.as_mut(), _env, _info, entry_fee, join_duration, turn_duration).unwrap();
            let mock_game =Game {
                players: Vec::new(),
                pot: Uint128::zero(),
                entry_fee,
                game_finished: false,
                join_duration,
                turn_duration,
                last_draw_time: join_duration,
                numbers: HashMap::new()
            };
            println!("{:?}", mock_game);
            assert_eq!(
                game,
                mock_game
            );
        }
        

        #[test]
        fn test_join_game(){
            let addr1 = String::from("addr1");
            let info = mock_info(addr1.as_ref(), &coins(100,"umlg"));
            let game_id = 0;
            let mut deps =  mock_dependencies();
            let _info = mock_info("creator", &[]);
            let _env = mock_env();
            let env  = mock_env();
            let entry_fee = Uint128::new(100);
            let join_duration = env.block.time.plus_seconds(1000);
            let turn_duration = Uint64::new(1000);
            let msg = InstantiateMsg {
                Denom : String::from("umlg"),
            };

            _do_instansiate(deps.as_mut(), env.clone(), info.clone(), msg);
            
            create_game(deps.as_mut(), _env, _info, entry_fee, join_duration, turn_duration).unwrap();
            
            let res = join_game(deps.as_mut(), env, info, game_id).unwrap();
            let events = res.attributes;
            let expected_event = Attribute::new("address", addr1.as_str());
            let actual_event = vec![expected_event];
            assert_eq!(events, actual_event);
        }


        #[test]
        fn test_board_if_game_not_started (){
            let mut deps = mock_dependencies();
            let env =  mock_env();
            let info = mock_info("creator", &[]);
            let game_id = 0;
            let res  = get_board(deps.as_mut(), env, info, game_id).unwrap_err();
            assert_eq!(res ,StdError::generic_err("game is not started yet"));
        }

        #[test]
        fn get_board_function (){
            let mut deps = mock_dependencies();
            let addr1 = String::from("addr1");

            let env =  mock_env();
            let info = mock_info(addr1.as_ref(), &coins(100,"umlg"));
            let game_id = 0;

            let res = get_board(deps.as_mut(), env, info, game_id).unwrap();
        }

        #[test]
        fn test_draw_number() {
            let addr1 = String::from("addr1");
            let info = mock_info(addr1.as_ref(), &coins(100,"umlg"));
            let mut deps =  mock_dependencies();
            let _info = mock_info("creator", &[]);
            let _env = mock_env();
            let env  = mock_env();
            let entry_fee = Uint128::new(100);
            let join_duration = env.block.time.plus_seconds(0);
            let turn_duration = Uint64::new(0);
            let msg = InstantiateMsg {
                Denom : String::from("umlg"),
            };
            let _game_id = 1;

            _do_instansiate(deps.as_mut(), env.clone(), info.clone(), msg);
            
            create_game(deps.as_mut(), _env, _info, entry_fee, join_duration, turn_duration).unwrap();
            
            let number = draw_number(deps.as_mut() , env, info, _game_id).unwrap();
            
            println!("{}", number);
        }
    }
}