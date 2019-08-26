#![feature(async_await, try_trait)]

mod board;
mod cells;
mod player;
mod position;

use board::GameBoardShotResult;
use player::Player;

const GAME_BOARD_SIZE: u8 = 10;

#[derive(Debug, Copy, Clone)]
pub struct InvalidInputError;

#[derive(structopt::StructOpt)]
struct Args {
    player1: std::path::PathBuf,
    player2: std::path::PathBuf,
}

#[derive(Debug, Copy, Clone)]
enum GameResult {
    Draw,
    Player1Win,
    Player2Win,
}

impl GameResult {
    fn from_results<T, E>(
        player1_result: Result<T, E>,
        player2_result: Result<T, E>,
    ) -> Result<Self, (T, T)> {
        match (player1_result, player2_result) {
            (Ok(player1_value), Ok(player2_value)) => Err((player1_value, player2_value)),
            (Err(_), Err(_)) => Ok(Self::Draw),
            (Ok(_), Err(_)) => Ok(Self::Player1Win),
            (Err(_), Ok(_)) => Ok(Self::Player2Win),
        }
    }

    fn print(self) {
        match self {
            Self::Draw => {
                println!("Draw!");
            }
            Self::Player1Win => {
                println!("Player 1 won the game!");
            }
            Self::Player2Win => {
                println!("Player 2 won the game!");
            }
        }
    }
}

#[paw::main]
#[tokio::main]
async fn main(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let game_result = play(args).await;
    game_result.print();
    Ok(())
}

async fn play(args: Args) -> GameResult {
    let player1 = Player::init(&args.player1).await;
    let player2 = Player::init(&args.player2).await;

    let (mut player1, mut player2) = match GameResult::from_results(player1, player2) {
        Ok(game_result) => return game_result,
        Err((player1, player2)) => (player1, player2),
    };

    println!("{}", player1.map_mut().to_string());
    println!("{}", player2.map_mut().to_string());

    let game_result = start_battle(&mut player1, &mut player2).await;

    eprintln!("Player 1 map: {:?}", player1.map_mut());
    eprintln!("Player 2 map: {:?}", player2.map_mut());

    game_result
}

async fn start_battle(player1: &mut Player, player2: &mut Player) -> GameResult {
    loop {
        loop {
            if let Some(shot_position) = player1.next_shot_position().await {
                let shot_result = player2.map_mut().shoot(shot_position);
                player1.reply_shot_result(shot_result).await;
                println!("1 {} {} {}", shot_position.x() + 1, shot_position.y() + 1, shot_result.as_str());
                if let GameBoardShotResult::Miss = shot_result {
                    break;
                }
                if player2.map_mut().hits_left() == 0 {
                    return GameResult::Player1Win;
                }
            } else {
                return GameResult::Player2Win;
            }
        }

        loop {
            if let Some(shot_position) = player2.next_shot_position().await {
                let shot_result = player1.map_mut().shoot(shot_position);
                player2.reply_shot_result(shot_result).await;
                println!("2 {} {} {}", shot_position.x() + 1, shot_position.y() + 1, shot_result.as_str());
                if let GameBoardShotResult::Miss = shot_result {
                    break;
                }
                if player1.map_mut().hits_left() == 0 {
                    return GameResult::Player2Win;
                }
            } else {
                return GameResult::Player1Win;
            }
        }
    }
}
