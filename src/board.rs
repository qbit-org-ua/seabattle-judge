use std::convert::TryFrom;

use crate::cells::{GameBoardCell, GameBoardCellState};
use crate::InvalidInputError;

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

#[derive(Debug, Copy, Clone)]
pub struct Position {
    x: std::num::NonZeroU8,
    y: std::num::NonZeroU8,
}

impl std::str::FromStr for Position {
    type Err = InvalidInputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let coords: Vec<&str> = s.split_whitespace().collect();

        let parse = |value: &str| {
            value.parse().map_err(|_| InvalidInputError {}).and_then(
                |value: std::num::NonZeroU8| {
                    if value.get() >= 1 && value.get() <= 10 {
                        Ok(value)
                    } else {
                        Err(InvalidInputError {})
                    }
                },
            )
        };
        Ok(Position {
            x: parse(coords[0])?,
            y: parse(coords[1])?,
        })
    }
}

const GAME_BOARD_SIZE: usize = 10;

#[derive(Clone)]
pub struct GameBoard {
    inner: Vec<Vec<GameBoardCell>>,
    hits_left: u8,
}

impl GameBoard {
    pub fn from_lines<Item, I>(lines: I) -> Result<Self, InvalidInputError>
    where
        Item: AsRef<str>,
        I: Iterator<Item = Item>,
    {
        let inner = lines
            .map(|line| {
                line.as_ref()
                    .chars()
                    .map(GameBoardCell::try_from)
                    .collect::<Result<Vec<GameBoardCell>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;

        if inner.len() != 10
            || inner
                .iter()
                .filter(|line| line.len() != 10)
                .next()
                .is_some()
        {
            return Err(InvalidInputError {});
        }

        let mut ships_count: [u8; 5] = [0, 4, 3, 2, 1];
        let hits_left = ships_count
            .iter()
            .zip(0..5)
            .map(|(ship_size, count)| ship_size * count)
            .sum();

        for (y, line) in inner.iter().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                if y > 0 {
                    if !cell.is_empty()
                        && ((x > 0 && !inner[y - 1][x - 1].is_empty())
                            || (x < GAME_BOARD_SIZE - 1 && !inner[y - 1][x + 1].is_empty()))
                    {
                        return Err(InvalidInputError {});
                    }
                    if !inner[y - 1][x].is_empty() {
                        continue;
                    }
                }
                if x > 0 && !inner[y][x - 1].is_empty() {
                    continue;
                }
                let vertical_ship_size = inner[y..GAME_BOARD_SIZE]
                    .iter()
                    .take_while(|line| line[x].is_ship())
                    .count();
                let horizontal_ship_size = inner[y][x..GAME_BOARD_SIZE]
                    .iter()
                    .take_while(|cell| cell.is_ship())
                    .count();
                let ship_size = if vertical_ship_size == horizontal_ship_size {
                    match vertical_ship_size {
                        0 => continue,
                        1 => 1,
                        _ => return Err(InvalidInputError {}),
                    }
                } else {
                    if vertical_ship_size == 1 {
                        horizontal_ship_size
                    } else if horizontal_ship_size == 1 {
                        vertical_ship_size
                    } else {
                        return Err(InvalidInputError {});
                    }
                };
                if ship_size >= ships_count.len() {
                    return Err(InvalidInputError {});
                }
                ships_count[ship_size] = ships_count[ship_size]
                    .checked_sub(1)
                    .ok_or_else(|| InvalidInputError {})?;
            }
        }
        if ships_count.iter().sum::<u8>() > 0 {
            return Err(InvalidInputError {});
        }
        Ok(Self { inner, hits_left })
    }

    pub fn shoot(&mut self, position: Position) -> GameBoardShotResult {
        let cell: &mut GameBoardCell =
            &mut self.inner[usize::from(position.y.get() - 1)][usize::from(position.x.get() - 1)];
        if let GameBoardCell::Ship(GameBoardCellState::NonShot) = cell {
            self.hits_left -= 1;
            *cell = GameBoardCell::Ship(GameBoardCellState::Shot);
            if true {
                // TODO
                GameBoardShotResult::Hit
            } else {
                GameBoardShotResult::Sunk
            }
        } else {
            if let GameBoardCell::Empty(GameBoardCellState::NonShot) = cell {
                *cell = GameBoardCell::Empty(GameBoardCellState::Shot);
            }
            GameBoardShotResult::Miss
        }
    }

    pub fn hits_left(&self) -> u8 {
        self.hits_left
    }
}

impl std::fmt::Debug for GameBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GameBoard ({} hits left):\n{}",
            self.hits_left,
            self.inner
                .iter()
                .map(|line| {
                    line.iter()
                        .copied()
                        .map(GameBoardCell::into)
                        .chain("\n".chars())
                        .collect::<String>()
                })
                .collect::<String>()
        )
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
