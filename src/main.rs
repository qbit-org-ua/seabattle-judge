#![feature(async_await)]

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

#[paw::main]
#[tokio::main]
async fn main(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let mut player1 = Player::init(&args.player1).unwrap();
    let mut player2 = Player::init(&args.player2).unwrap();

    let mut player1_map = player1.read_map().await.unwrap();
    println!("PLAYER1 MAP: {:?}", player1_map);
    let mut player2_map = player2.read_map().await.unwrap();
    println!("PLAYER2 MAP: {:?}", player2_map);

    'game: loop {
        loop {
            if let Some(shot_position) = player1.next_shot_position().await {
                let shot_result = player2_map.shoot(shot_position);
                player1.reply_shot_result(shot_result).await;
                if let GameBoardShotResult::Miss = shot_result {
                    break;
                }
                if player2_map.hits_left() == 0 {
                    println!("Player 1 won the game!");
                    break 'game;
                }
            } else {
                break 'game;
            }
        }

        loop {
            if let Some(shot_position) = player2.next_shot_position().await {
                let shot_result = player1_map.shoot(shot_position);
                player2.reply_shot_result(shot_result).await;
                if let GameBoardShotResult::Miss = shot_result {
                    break;
                }
                if player1_map.hits_left() == 0 {
                    println!("Player 2 won the game!");
                    break 'game;
                }
            } else {
                break 'game;
            }
        }
    }
    eprintln!("PLAYER1 MAP: {:?}", player1_map);
    eprintln!("PLAYER2 MAP: {:?}", player2_map);

    Ok(())
}
