use std::convert::TryFrom;

use crate::InvalidInputError;

#[derive(Debug, Copy, Clone)]
pub enum GameBoardCellState {
    NonShot,
    Shot,
}

impl Default for GameBoardCellState {
    fn default() -> Self {
        Self::NonShot
    }
}

#[derive(Debug, Copy, Clone)]
pub enum GameBoardCell {
    Empty(GameBoardCellState),
    Ship(GameBoardCellState),
}

impl GameBoardCell {
    pub fn is_empty(self) -> bool {
        match self {
            Self::Empty(_) => true,
            _ => false,
        }
    }

    pub fn is_ship(self) -> bool {
        match self {
            Self::Ship(_) => true,
            _ => false,
        }
    }

    pub fn is_shot(self) -> bool {
        match self {
            Self::Ship(GameBoardCellState::Shot) | Self::Empty(GameBoardCellState::Shot) => true,
            _ => false,
        }
    }
}

impl TryFrom<char> for GameBoardCell {
    type Error = InvalidInputError;

    fn try_from(c: char) -> Result<Self, InvalidInputError> {
        match c {
            '_' => Ok(Self::Empty(GameBoardCellState::NonShot)),
            '#' => Ok(Self::Ship(GameBoardCellState::NonShot)),
            _ => Err(InvalidInputError {}),
        }
    }
}

impl From<GameBoardCell> for char {
    fn from(cell: GameBoardCell) -> Self {
        match cell {
            GameBoardCell::Empty(GameBoardCellState::NonShot) => '_',
            GameBoardCell::Empty(GameBoardCellState::Shot) => 'O',
            GameBoardCell::Ship(GameBoardCellState::NonShot) => '#',
            GameBoardCell::Ship(GameBoardCellState::Shot) => '$',
        }
    }
}
