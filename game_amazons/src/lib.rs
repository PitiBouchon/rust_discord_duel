#![feature(let_chains)]

pub mod config;
pub mod input;
pub mod pos;

use crate::config::AmazonsConfig;
use crate::input::PlayerInput;
use duel_game::{DiscordDuelGame, PlayerTurn};
use pos::Pos;
use rand::prelude::IteratorRandom;
use std::fmt::{Display, Formatter};

#[derive(Debug, Eq, PartialEq)]
pub enum GameCell {
    Empty,
    Amazon1,
    Amazon2,
    Arrow,
}

#[derive(Debug)]
pub enum GameError {
    InputOutOfBounds,
    InputInvalidPosition,
    InvalidTravel,
    InvalidArrowTravel,
}

impl std::error::Error for GameError {}

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::InputInvalidPosition => write!(f, "Invalid position"),
            GameError::InputOutOfBounds => write!(f, "Input is out of bounds"),
            GameError::InvalidTravel => write!(f, "Invalid travel"),
            GameError::InvalidArrowTravel => write!(f, "Invalid arrow travel"),
        }
    }
}

pub struct AmazonsGame(Vec<Vec<GameCell>>);

impl DiscordDuelGame for AmazonsGame {
    type Config = AmazonsConfig;
    type Input = PlayerInput;
    type GameError = GameError;

    fn new(config: Self::Config) -> Self {
        let mut rng = rand::thread_rng();

        let queen1_xpos: Vec<usize> = (0..config.width).choose_multiple(&mut rng, config.queens);
        let queen2_xpos: Vec<usize> = (0..config.width).choose_multiple(&mut rng, config.queens);
        let queen1_ypos: Vec<usize> = (0..config.height).choose_multiple(&mut rng, config.queens);
        let queen2_ypos: Vec<usize> = (0..config.height).choose_multiple(&mut rng, config.queens);

        let mut grid: Vec<Vec<GameCell>> = (0..config.width)
            .map(|_| {
                (0..config.height)
                    .map(|_| GameCell::Empty)
                    .collect::<Vec<_>>()
            })
            .collect();

        for (x, y) in queen1_xpos.into_iter().zip(queen1_ypos) {
            grid[x][y] = GameCell::Amazon1;
        }

        for (x, y) in queen2_xpos.into_iter().zip(queen2_ypos) {
            grid[x][y] = GameCell::Amazon2;
        }

        Self(grid)
    }

    fn to_console_player1(&self) -> String {
        self.0
            .iter()
            .map(|column| {
                column
                    .iter()
                    .map(|cell| match cell {
                        GameCell::Empty => "_".to_string(),
                        GameCell::Amazon1 => "*".to_string(),
                        GameCell::Amazon2 => "+".to_string(),
                        GameCell::Arrow => "@".to_string(),
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn to_console_player2(&self) -> String {
        self.0
            .iter()
            .map(|column| {
                column
                    .iter()
                    .map(|cell| match cell {
                        GameCell::Empty => "_".to_string(),
                        GameCell::Amazon1 => "+".to_string(),
                        GameCell::Amazon2 => "*".to_string(),
                        GameCell::Arrow => "@".to_string(),
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn to_discord(&self) -> String {
        self.0
            .iter()
            .map(|column| {
                column
                    .iter()
                    .map(|cell| match cell {
                        GameCell::Empty => "⬛".to_string(),
                        GameCell::Amazon1 => "🐝".to_string(),
                        GameCell::Amazon2 => "🐨".to_string(),
                        GameCell::Arrow => "🧱".to_string(),
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    // Return true if the game is ended, false otherwise
    fn play(&mut self, player_input: Self::Input, n: PlayerTurn) -> Result<bool, Self::GameError> {
        let from_cell = self
            .get_cell(player_input.from)
            .ok_or(GameError::InputOutOfBounds)?;

        match n {
            PlayerTurn::Player1 => {
                if *from_cell != GameCell::Amazon1 {
                    return Err(GameError::InputInvalidPosition);
                }
            }
            PlayerTurn::Player2 => {
                if *from_cell != GameCell::Amazon2 {
                    return Err(GameError::InputInvalidPosition);
                }
            }
        }

        if player_input.from == player_input.to {
            return Err(GameError::InputInvalidPosition);
        }

        let (dir_x, dir_y) =
            get_dir(player_input.from, player_input.to).ok_or(GameError::InvalidTravel)?;
        assert!(dir_x != 0 || dir_y != 0);

        let mut from_pos = player_input.from;
        while from_pos != player_input.to && let Some(new_pos) = from_pos.shift(dir_x, dir_y) {
            if new_pos == from_pos {
                return Err(GameError::InvalidTravel);
            }
            if *self.get_cell(new_pos).ok_or(GameError::InvalidTravel)? != GameCell::Empty {
                return Err(GameError::InvalidTravel);
            }
            from_pos = new_pos;
        }

        let (dir_x, dir_y) =
            get_dir(player_input.to, player_input.arrow).ok_or(GameError::InvalidArrowTravel)?;
        assert!(dir_x != 0 || dir_y != 0);

        let mut to_pos = player_input.arrow;
        while to_pos != player_input.arrow && let Some(new_pos) = from_pos.shift(dir_x, dir_y) {
            if new_pos == to_pos {
                return Err(GameError::InvalidArrowTravel);
            }
            if *self
                .get_cell(new_pos)
                .ok_or(GameError::InvalidArrowTravel)?
                != GameCell::Empty
            {
                return Err(GameError::InvalidArrowTravel);
            }
            to_pos = new_pos;
        }

        *self
            .get_mut_cell(player_input.from)
            .ok_or(GameError::InputOutOfBounds)? = GameCell::Empty;
        *self
            .get_mut_cell(player_input.to)
            .ok_or(GameError::InputOutOfBounds)? = if n == PlayerTurn::Player1 {
            GameCell::Amazon1
        } else {
            GameCell::Amazon2
        };
        *self
            .get_mut_cell(player_input.arrow)
            .ok_or(GameError::InputOutOfBounds)? = GameCell::Arrow;

        Ok(false)
    }
}

impl AmazonsGame {
    fn get_cell(&self, pos: Pos) -> Option<&GameCell> {
        self.0.get(pos.x)?.get(pos.y)
    }

    fn get_mut_cell(&mut self, pos: Pos) -> Option<&mut GameCell> {
        self.0.get_mut(pos.x)?.get_mut(pos.y)
    }
}

fn get_dir(from: Pos, to: Pos) -> Option<(isize, isize)> {
    let dir = (
        (to.x as isize - from.x as isize).signum(),
        (to.y as isize - from.y as isize).signum(),
    );

    if dir.0 != 0 && dir.1 != 0 {
        if (to.x as isize - from.x as isize).abs() != (to.y as isize - from.y as isize).abs() {
            return None;
        }
    }

    if dir.0 == 0 && dir.1 == 0 {
        return None;
    }

    Some(dir)
}
