use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::CommandDataOption;
use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PlayerTurn {
    Player1,
    Player2,
}

impl PlayerTurn {
    pub fn next(self) -> Self {
        match self {
            Self::Player1 => Self::Player2,
            Self::Player2 => Self::Player1,
        }
    }
}

pub trait DiscordDuelGame: Send + Sync {
    type Config: DiscordConfig + Send + Sync + Display;
    type Input: FromStr + Send + Sync;
    type GameError: Error;

    fn new(config: Self::Config) -> Self;

    fn to_console_player1(&self) -> String;
    fn to_console_player2(&self) -> String;

    fn to_discord(&self) -> String;

    fn play(&mut self, player_input: Self::Input, n: PlayerTurn) -> Result<bool, Self::GameError>;
}

pub trait DiscordConfig {
    fn create_command(option: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;
    fn from_options(options: &[CommandDataOption]) -> Self;
}
