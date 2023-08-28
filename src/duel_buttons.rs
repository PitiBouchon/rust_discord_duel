use anyhow::{Error, Result};
use serenity::model::prelude::message_component::MessageComponentInteraction;

pub mod play;
pub mod quit;

fn parse_first_line_game_id(command: &MessageComponentInteraction) -> Result<usize> {
    const GAME_MSG: &str = "# Game ";
    let first_line = command
        .message
        .content
        .lines()
        .next()
        .ok_or(Error::msg("Cannot get first line"))?;
    if first_line.len() < GAME_MSG.len() {
        return Err(Error::msg(format!(
            "First line to short ({} characters)",
            first_line.len()
        )));
    }
    Ok(first_line[GAME_MSG.len()..].parse()?)
}
