#![feature(async_await)]

use futures::sink::SinkExt;
use futures::stream::StreamExt;
use std::process::{Command, Stdio};
use tokio::codec::{FramedRead, FramedWrite, LinesCodec};
use tokio_process::CommandExt;

mod board;
mod cells;

use board::{GameBoard, GameBoardShotResult};

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
    let mut player1_cmd = Command::new(args.player1);
    player1_cmd.stdin(Stdio::piped());
    player1_cmd.stdout(Stdio::piped());

    let mut player1_child = player1_cmd.spawn_async().expect("failed to spawn command");

    let player1_stdin = player1_child
        .stdin()
        .take()
        .expect("child did not have a handle to stdin");
    let player1_stdout = player1_child
        .stdout()
        .take()
        .expect("child did not have a handle to stdout");

    let mut player1_writer = FramedWrite::new(player1_stdin, LinesCodec::new());
    let mut player1_reader = FramedRead::new(player1_stdout, LinesCodec::new());

    let mut player2_cmd = Command::new(args.player2);
    player2_cmd.stdin(Stdio::piped());
    player2_cmd.stdout(Stdio::piped());

    let mut player2_child = player2_cmd.spawn_async().expect("failed to spawn command");

    let player2_stdin = player2_child
        .stdin()
        .take()
        .expect("child did not have a handle to stdin");
    let player2_stdout = player2_child
        .stdout()
        .take()
        .expect("child did not have a handle to stdout");

    let mut player2_writer = FramedWrite::new(player2_stdin, LinesCodec::new());
    let mut player2_reader = FramedRead::new(player2_stdout, LinesCodec::new());

    // Ensure the child process is spawned in the runtime so it can
    // make progress on its own while we await for any output.
    tokio::spawn(async {
        let status = player1_child
            .await
            .expect("child process encountered an error");

        println!("child status was: {}", status);
    });

    tokio::spawn(async {
        let status = player2_child
            .await
            .expect("child process encountered an error");

        println!("child status was: {}", status);
    });

    use tokio::stream::StreamExt;
    let mut player1_map_stream = player1_reader
        .by_ref()
        .chunks(10)
        .timeout(std::time::Duration::from_secs(1));
    let mut player1_map = if let Some(Ok(lines)) = player1_map_stream.next().await {
        GameBoard::from_lines(lines.into_iter().filter_map(|line| line.ok()))
    } else {
        Err(InvalidInputError {})
    }
    .unwrap();
    println!("PLAYER1 MAP: {:?}", player1_map);

    let mut player2_map_stream = player2_reader
        .by_ref()
        .chunks(10)
        .timeout(std::time::Duration::from_secs(1));
    let mut player2_map = if let Some(Ok(lines)) = player2_map_stream.next().await {
        GameBoard::from_lines(lines.into_iter().filter_map(|line| line.ok()))
    } else {
        Err(InvalidInputError {})
    }
    .unwrap();
    println!("PLAYER2 MAP: {:?}", player1_map);

    'game: loop {
        loop {
            if let Some(Ok(line)) = player1_reader.next().await {
                let shot_position = line.parse::<board::Position>().unwrap();
                let shot_result = player2_map.shoot(shot_position);
                player1_writer
                    .send(shot_result.as_str().to_owned())
                    .await
                    .unwrap();
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
            if let Some(Ok(line)) = player2_reader.next().await {
                let shot_position = line.parse::<board::Position>().unwrap();
                let shot_result = player1_map.shoot(shot_position);
                player2_writer
                    .send(shot_result.as_str().to_owned())
                    .await
                    .unwrap();
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
