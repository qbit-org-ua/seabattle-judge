use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::process::Stdio;
use tokio::codec::{FramedRead, FramedWrite, LinesCodec};
use tokio_process::Command;

use crate::board::{GameBoard, GameBoardShotResult};
use crate::position::Position;
use crate::InvalidInputError;

pub struct Player {
    reader: FramedRead<tokio_process::ChildStdout, LinesCodec>,
    writer: FramedWrite<tokio_process::ChildStdin, LinesCodec>,
    map: GameBoard,
}

impl Player {
    pub fn map_mut(&mut self) -> &mut GameBoard {
        &mut self.map
    }

    pub async fn init(player_exe: &std::path::Path) -> Result<Self, InvalidInputError> {
        let mut player_cmd = Command::new(player_exe);
        player_cmd.stdin(Stdio::piped());
        player_cmd.stdout(Stdio::piped());

        let mut child = player_cmd.spawn().expect("failed to spawn command");

        let player_stdin = child
            .stdin()
            .take()
            .expect("child did not have a handle to stdin");
        let player_stdout = child
            .stdout()
            .take()
            .expect("child did not have a handle to stdout");

        let mut reader = FramedRead::new(player_stdout, LinesCodec::new());
        let writer = FramedWrite::new(player_stdin, LinesCodec::new());

        // make progress on its own while we await for any output.
        tokio::spawn(async {
            let status = child.await.expect("child process encountered an error");

            eprintln!("child status was: {}", status);
        });

        let map = GameBoard::read(&mut reader).await?;
        Ok(Self {
            reader,
            writer,
            map,
        })
    }

    pub async fn next_shot_position(&mut self) -> Option<Position> {
        match self.reader.next().await? {
            Ok(line) => match line.parse::<Position>() {
                Ok(position) => Some(position),
                Err(err) => {
                    eprintln!("Next shot position failed due to: {:?}", err);
                    None
                }
            },
            Err(err) => {
                eprintln!("Next shot position failed due to: {:?}", err);
                None
            }
        }
    }

    pub async fn reply_shot_result(&mut self, shot_result: GameBoardShotResult) {
        let _ = self.writer
            .send(shot_result.as_str().to_owned())
            .await;
    }
}
