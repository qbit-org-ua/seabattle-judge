use std::convert::TryFrom;

use crate::{InvalidInputError, GAME_BOARD_SIZE};

#[derive(Debug, Copy, Clone)]
pub struct Position {
    yx: u8,
}

impl Position {
    pub fn top_left() -> Self {
        Self { yx: 0 }
    }

    pub fn bottom_right() -> Self {
        Self {
            yx: GAME_BOARD_SIZE * GAME_BOARD_SIZE - 1,
        }
    }

    pub fn x(self) -> u8 {
        self.yx % GAME_BOARD_SIZE
    }

    pub fn y(self) -> u8 {
        self.yx / GAME_BOARD_SIZE
    }

    pub fn yx(self) -> u8 {
        self.yx
    }

    pub fn get_left(self) -> Option<Self> {
        self.iter_left().nth(1)
    }

    pub fn get_right(self) -> Option<Self> {
        self.iter_right().nth(1)
    }

    pub fn get_above(self) -> Option<Self> {
        self.iter_above().nth(1)
    }

    pub fn iter(self) -> PositionIter {
        PositionIter {
            start: self,
            end: Position::bottom_right(),
        }
    }

    pub fn iter_left(mut self) -> impl Iterator<Item = Position> {
        let x = self.x();
        self.yx = self.y() * GAME_BOARD_SIZE;
        self.iter().take(usize::from(x + 1)).rev()
    }

    pub fn iter_right(self) -> impl Iterator<Item = Position> {
        self.iter()
            .take(usize::from(GAME_BOARD_SIZE - self.yx % GAME_BOARD_SIZE))
    }

    pub fn iter_above(mut self) -> impl Iterator<Item = Position> {
        let y = self.y();
        self.yx = self.x();
        self.iter()
            .step_by(usize::from(GAME_BOARD_SIZE))
            .take(usize::from(y + 1))
            .rev()
    }

    pub fn iter_below(self) -> impl Iterator<Item = Position> {
        self.iter().step_by(usize::from(GAME_BOARD_SIZE))
    }
}

impl std::str::FromStr for Position {
    type Err = InvalidInputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let coords: Vec<&str> = s.split_whitespace().collect();

        let parse = |value: &str| {
            value
                .parse()
                .map_err(|_| InvalidInputError {})
                .and_then(|value: u8| {
                    if value >= 1 && value <= GAME_BOARD_SIZE {
                        Ok(value)
                    } else {
                        Err(InvalidInputError {})
                    }
                })
        };
        Ok(Position {
            yx: (parse(coords[1])? - 1) * GAME_BOARD_SIZE + parse(coords[0])? - 1,
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PositionIter {
    start: Position,
    end: Position,
}

impl Iterator for PositionIter {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start.yx <= self.end.yx {
            let position = self.start;
            self.start.yx += 1;
            Some(position)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.start.yx <= self.end.yx {
            let size = usize::from(self.end.yx - self.start.yx + 1);
            (size, Some(size))
        } else {
            (0, Some(0))
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.start.yx <= self.end.yx {
            self.start.yx += u8::try_from(n).unwrap();
            if self.start.yx <= self.end.yx {
                let position = self.start;
                self.start.yx += 1;
                return Some(position);
            }
        }
        None
    }
}

impl std::iter::ExactSizeIterator for PositionIter {}

impl DoubleEndedIterator for PositionIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start.yx <= self.end.yx {
            let position = self.end;
            if self.end.yx > 0 {
                self.end.yx -= 1;
            } else {
                self.start.yx += 1;
            }
            Some(position)
        } else {
            None
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if self.start.yx <= self.end.yx {
            let n = u8::try_from(n).unwrap();
            if self.end.yx < n {
                self.start.yx = 1;
                self.end.yx = 0;
            } else {
                self.end.yx -= n;
                if self.start.yx <= self.end.yx {
                    let position = self.end;
                    if self.end.yx > 0 {
                        self.end.yx -= 1;
                    } else {
                        self.start.yx += 1;
                    }
                    return Some(position);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn test_Position_iter() {
        let mut pos_iter = Position::top_left().iter();
        let pos = pos_iter.next();
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().yx(), 0);
        let mut pos_iter = pos_iter.skip(usize::from(GAME_BOARD_SIZE * GAME_BOARD_SIZE) - 2);
        let pos = pos_iter.next();
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().yx(), GAME_BOARD_SIZE * GAME_BOARD_SIZE - 1);
    }

    #[test]
    fn test_Position_iter_back() {
        let mut pos_iter = Position::top_left().iter().rev();
        let pos = pos_iter.next();
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().yx(), GAME_BOARD_SIZE * GAME_BOARD_SIZE - 1);
        let mut pos_iter = pos_iter.skip(usize::from(GAME_BOARD_SIZE * GAME_BOARD_SIZE) - 2);
        let pos = pos_iter.next();
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().yx(), 0);
    }

    #[test]
    fn test_Position_iter_left() {
        let mut pos_iter = Position::bottom_right().iter_left();
        let pos = pos_iter.next();
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().yx(), GAME_BOARD_SIZE * GAME_BOARD_SIZE - 1);
        let mut pos_iter = pos_iter.skip(usize::from(GAME_BOARD_SIZE) - 2);
        let pos = pos_iter.next();
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().yx(), GAME_BOARD_SIZE * (GAME_BOARD_SIZE - 1));
        assert!(pos_iter.next().is_none());
    }

    #[test]
    fn test_Position_get_left() {
        assert!(Position::top_left().get_left().is_none());
        assert!(Position::top_left()
            .iter()
            .nth(usize::from(GAME_BOARD_SIZE))
            .unwrap()
            .get_left()
            .is_none());
        assert_eq!(
            Position::bottom_right().get_left().map(|x| x.yx()),
            Some(GAME_BOARD_SIZE * GAME_BOARD_SIZE - 2)
        );
        let pos = Position::top_left().iter().nth(1).unwrap();
        assert_eq!(pos.get_left().map(|x| x.yx()), Some(0));
    }
}
