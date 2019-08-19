use futures::sink::SinkExt;
use futures::stream::StreamExt;
use std::process::{Command, Stdio};
use tokio::codec::{FramedRead, FramedWrite, LinesCodec};
use tokio_process::CommandExt;

use crate::board::{GameBoard, GameBoardShotResult};
use crate::position::Position;
use crate::InvalidInputError;

pub struct Player {
    reader: FramedRead<tokio_process::ChildStdout, LinesCodec>,
    writer: FramedWrite<tokio_process::ChildStdin, LinesCodec>,
}

impl Player {
    pub fn init(player_exe: &std::path::Path) -> Result<Self, ()> {
        let mut player_cmd = Command::new(player_exe);
        player_cmd.stdin(Stdio::piped());
        player_cmd.stdout(Stdio::piped());

        let mut child = player_cmd.spawn_async().expect("failed to spawn command");

        let player_stdin = child
            .stdin()
            .take()
            .expect("child did not have a handle to stdin");
        let player_stdout = child
            .stdout()
            .take()
            .expect("child did not have a handle to stdout");

        let reader = FramedRead::new(player_stdout, LinesCodec::new());
        let writer = FramedWrite::new(player_stdin, LinesCodec::new());

        // make progress on its own while we await for any output.
        tokio::spawn(async {
            let status = child.await.expect("child process encountered an error");

            println!("child status was: {}", status);
        });

        Ok(Self { reader, writer })
    }

    pub async fn read_map(&mut self) -> Result<GameBoard, InvalidInputError> {
        use tokio::stream::StreamExt;
        let mut player_map_stream = self
            .reader
            .by_ref()
            .chunks(10)
            .timeout(std::time::Duration::from_secs(1));
        if let Some(Ok(lines)) = player_map_stream.next().await {
            GameBoard::from_lines(lines.into_iter().filter_map(|line| line.ok()))
        } else {
            Err(InvalidInputError {})
        }
    }

    pub async fn next_shot_position(&mut self) -> Option<Position> {
        match self.reader.next().await? {
            Ok(line) => Some(line.parse::<Position>().unwrap()),
            Err(err) => {
                eprintln!("Next shot position failed due to: {:?}", err);
                None
            }
        }
    }

    pub async fn reply_shot_result(&mut self, shot_result: GameBoardShotResult) {
        self.writer
            .send(shot_result.as_str().to_owned())
            .await
            .unwrap();
    }
}
