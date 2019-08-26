use std::convert::TryFrom;

use futures_util::stream::StreamExt as _;
use tokio::codec::FramedRead;
use tokio::stream::StreamExt as _;

use crate::cells::{GameBoardCell, GameBoardCellState};
use crate::position::Position;
use crate::{InvalidInputError, GAME_BOARD_SIZE};

#[derive(Debug, Copy, Clone)]
pub enum GameBoardShotResult {
    Miss,
    Hit,
    Sunk,
}

impl GameBoardShotResult {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Miss => "miss",
            Self::Hit => "hit",
            Self::Sunk => "sunk",
        }
    }
}

#[derive(Clone)]
pub struct GameBoard {
    inner: arrayvec::ArrayVec<[GameBoardCell; (GAME_BOARD_SIZE * GAME_BOARD_SIZE) as usize]>,
    hits_left: u8,
}

impl GameBoard {
    fn get_index(pos: Position) -> usize {
        usize::from(pos.yx())
    }
    pub fn get(&self, pos: Position) -> GameBoardCell {
        self.inner[Self::get_index(pos)]
    }

    pub fn get_mut(&mut self, pos: Position) -> &mut GameBoardCell {
        &mut self.inner[Self::get_index(pos)]
    }

    pub async fn read<T, D>(reader: &mut FramedRead<T, D>) -> Result<Self, InvalidInputError>
    where
        T: tokio::io::AsyncRead + Unpin,
        D: tokio::codec::Decoder + Unpin,
        <D as tokio::codec::Decoder>::Item: std::convert::AsRef<str>,
    {
        let mut player_map_stream = reader.chunks(10).timeout(std::time::Duration::from_secs(1));
        if let Some(Ok(lines)) = player_map_stream.next().await {
            GameBoard::from_lines(lines.into_iter().filter_map(|line| line.ok()))
        } else {
            Err(InvalidInputError {})
        }
    }

    pub fn from_lines<Item, I>(lines: I) -> Result<Self, InvalidInputError>
    where
        Item: AsRef<str>,
        I: Iterator<Item = Item>,
    {
        let mut ships_count: [u8; 5] = [0, 4, 3, 2, 1];
        let hits_left = ships_count
            .iter()
            .zip(0..5)
            .map(|(ship_size, count)| ship_size * count)
            .sum();

        // Parse string map into `GameBoardCell`s and validate shape
        let board = Self {
            inner: lines
                .take(10)
                .flat_map(|line| {
                    if line.as_ref().len() == usize::from(GAME_BOARD_SIZE) {
                        line.as_ref().chars().map(GameBoardCell::try_from).collect()
                    } else {
                        arrayvec::ArrayVec::from(
                            [Err(InvalidInputError {}); GAME_BOARD_SIZE as usize],
                        )
                    }
                })
                .collect::<Result<_, _>>()?,
            hits_left,
        };

        if board.inner.len() != usize::from(GAME_BOARD_SIZE * GAME_BOARD_SIZE) {
            return Err(InvalidInputError {});
        }

        // Validate amount of ships and their shape
        for position in Position::top_left().iter() {
            if let Some(position_above) = position.get_above() {
                // Detect ships that are touching by their corners
                if !board.get(position).is_empty()
                    && ![position_above.get_left(), position_above.get_right()]
                        .iter()
                        .flatten()
                        .all(|&pos| board.get(pos).is_empty())
                {
                    return Err(InvalidInputError {});
                }

                // If there is a non-empty cell above the current cell, we skip further checking as
                // we should have already accounted this cell when handling that one.
                if !board.get(position_above).is_empty() {
                    continue;
                }
            }
            // If there is a non-empty cell on the left of the current cell, we skip further
            // checking as we should have already accounted this cell when handling that one.
            if let Some(false) = position.get_left().map(|pos| board.get(pos).is_empty()) {
                continue;
            }

            let vertical_ship_size = position
                .iter_below()
                .take_while(|&pos| board.get(pos).is_ship())
                .count();
            let horizontal_ship_size = position
                .iter_right()
                .take_while(|&pos| board.get(pos).is_ship())
                .count();

            let ship_size = if vertical_ship_size == horizontal_ship_size {
                match vertical_ship_size {
                    0 => continue,
                    1 => 1,
                    _ => return Err(InvalidInputError {}),
                }
            } else if vertical_ship_size == 1 {
                horizontal_ship_size
            } else if horizontal_ship_size == 1 {
                vertical_ship_size
            } else {
                return Err(InvalidInputError {});
            };

            let ship_count = ships_count
                .get_mut(ship_size)
                .ok_or_else(|| InvalidInputError {})?;
            *ship_count = ship_count
                .checked_sub(1)
                .ok_or_else(|| InvalidInputError {})?;
        }

        if ships_count.iter().sum::<u8>() > 0 {
            Err(InvalidInputError {})
        } else {
            Ok(board)
        }
    }

    pub fn to_string(&self) -> String {
        self.inner
            .chunks(usize::from(GAME_BOARD_SIZE))
            .map(|line| {
                line.iter()
                    .copied()
                    .map(GameBoardCell::into)
                    .chain("\n".chars())
                    .collect::<String>()
            })
            .collect()
    }

    pub fn shoot(&mut self, position: Position) -> GameBoardShotResult {
        let cell: &mut GameBoardCell = self.get_mut(position);
        let shot_result = if let GameBoardCell::Ship(GameBoardCellState::NonShot) = cell {
            *cell = GameBoardCell::Ship(GameBoardCellState::Shot);

            fn iter_helper<'a>(
                board: &'a GameBoard,
                iter: impl Iterator<Item = Position> + 'a,
            ) -> impl Iterator<Item = GameBoardCell> + 'a {
                iter.skip(1)
                    .map(move |pos| board.get(pos))
                    .take_while(|cell| cell.is_ship())
            }

            if iter_helper(self, position.iter_left())
                .chain(iter_helper(self, position.iter_right()))
                .chain(iter_helper(self, position.iter_above()))
                .chain(iter_helper(self, position.iter_below()))
                .all(|cell| cell.is_shot())
            {
                GameBoardShotResult::Sunk
            } else {
                GameBoardShotResult::Hit
            }
        } else {
            if let GameBoardCell::Empty(GameBoardCellState::NonShot) = cell {
                *cell = GameBoardCell::Empty(GameBoardCellState::Shot);
            }
            GameBoardShotResult::Miss
        };
        if let GameBoardShotResult::Hit | GameBoardShotResult::Sunk = shot_result {
            self.hits_left -= 1;
        }
        shot_result
    }

    pub fn hits_left(&self) -> u8 {
        self.hits_left
    }
}

impl std::fmt::Debug for GameBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameBoard ({} hits left):\n{}", self.hits_left, self.to_string())
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn test_GameBoard_validation_ok() {
        assert!(GameBoard::from_lines(
            "\
             ####_###_# \
             _________# \
             _________# \
             __________ \
             _________# \
             _________# \
             __________ \
             #________# \
             _________# \
             #_#_#_##__ \
             "
            .split_whitespace()
        )
        .is_ok());
    }

    #[test]
    fn test_GameBoard_validation_of_invalid_size() {
        assert!(GameBoard::from_lines("".split_whitespace()).is_err());

        assert!(GameBoard::from_lines(
            "\
             __________ \
             __________ \
             __________ \
             __________ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             ___________ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             _________ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             ______ \
             ______ \
             ______ \
             ______ \
             ______ \
             ______ \
             ______ \
             ______ \
             ______ \
             ______ \
             "
            .split_whitespace()
        )
        .is_err());
    }

    #[test]
    fn test_GameBoard_validation_of_invalid_chars() {
        assert!(GameBoard::from_lines(
            "\
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             __________ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             ########## \
             ########## \
             ########## \
             ########## \
             ########## \
             ########## \
             ########## \
             ########## \
             ########## \
             ########## \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             !!!!!!!!!! \
             !!!!!!!!!! \
             !!!!!!!!!! \
             !!!!!!!!!! \
             !!!!!!!!!! \
             !!!!!!!!!! \
             !!!!!!!!!! \
             !!!!!!!!!! \
             !!!!!!!!!! \
             !!!!!!!!!! \
             "
            .split_whitespace()
        )
        .is_err());
    }

    #[test]
    fn test_GameBoard_validation_of_invalid_number_of_ships() {
        assert!(GameBoard::from_lines(
            "\
             ####______ \
             __________ \
             ###_###___ \
             __________ \
             ##_##_##__ \
             __________ \
             #_#_#_#___ \
             __________ \
             __________ \
             ####______ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             ####______ \
             __________ \
             ###_###___ \
             __________ \
             ##_##_##__ \
             __________ \
             #_#_#_#___ \
             __________ \
             __________ \
             ###_______ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             ####______ \
             __________ \
             ###_###___ \
             __________ \
             ##_##_##__ \
             __________ \
             #_#_#_#___ \
             __________ \
             __________ \
             ##________ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             ####______ \
             __________ \
             ###_###___ \
             __________ \
             ##_##_##__ \
             __________ \
             #_#_#_#___ \
             __________ \
             __________ \
             #_________ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             #_#_#_#__# \
             #_#_#____# \
             #_#___#__# \
             #___#____# \
             __#_#_#___ \
             __#_______ \
             __#_#_#___ \
             ____#_____ \
             __________ \
             __________ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             #_#_#_#__# \
             #_#_#____# \
             #_#___#__# \
             #___#_____ \
             __#_#_#___ \
             __#_______ \
             __#_#_#___ \
             ____#_____ \
             __________ \
             __________ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             #_#_#_#__# \
             #_#_#____# \
             #_#___#___ \
             #___#_____ \
             __#_#_#___ \
             __#_______ \
             __#_#_#___ \
             ____#_____ \
             __________ \
             __________ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             #_#_#_#__# \
             #_#_#_____ \
             #_#___#___ \
             #___#_____ \
             __#_#_#___ \
             __#_______ \
             __#_#_#___ \
             ____#_____ \
             __________ \
             __________ \
             "
            .split_whitespace()
        )
        .is_err());
    }

    #[test]
    fn test_GameBoard_validation_of_invalid_ship_shapes() {
        assert!(GameBoard::from_lines(
            "\
             ####______ \
             ###_###___ \
             __________ \
             __________ \
             ##_##_##__ \
             __________ \
             #_#_#_#___ \
             __________ \
             __________ \
             __________ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             ####______ \
             _______#__ \
             ###_###___ \
             __________ \
             ##_##_##__ \
             __________ \
             #_#_#_____ \
             __________ \
             __________ \
             __________ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             ####______ \
             __________ \
             ###_###___ \
             __________ \
             ##_##_##__ \
             __#_______ \
             __#_#_#___ \
             __________ \
             __________ \
             __________ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             ####______ \
             ___#______ \
             ___#______ \
             __________ \
             ###_______ \
             __________ \
             ##_##_##__ \
             __________ \
             #_#_#_#___ \
             __________ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             _#________ \
             ####______ \
             _#________ \
             __________ \
             ###_______ \
             __________ \
             ##_##_##__ \
             __________ \
             #_#_#_#___ \
             __________ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             ####______ \
             __________ \
             ###_###___ \
             __________ \
             ##_##_##__ \
             __________ \
             ___#_#_#__ \
             __#_______ \
             __________ \
             __________ \
             "
            .split_whitespace()
        )
        .is_err());

        assert!(GameBoard::from_lines(
            "\
             ####______ \
             __________ \
             ###_###___ \
             __________ \
             ##_##_##__ \
             __________ \
             _#_#_#_#__ \
             __________ \
             _#####____ \
             __________ \
             "
            .split_whitespace()
        )
        .is_err());
    }
}
