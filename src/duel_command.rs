use crate::handler::{GameInstance, Handler};
use anyhow::{Error, Result};
use duel_game::{DiscordConfig, DiscordDuelGame, PlayerTurn};
use serenity::client::Context;
use serenity::model::application::component::ButtonStyle;
use serenity::model::channel::{Attachment, ReactionType};
use serenity::model::prelude::application_command::{
    ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
};
use serenity::model::prelude::InteractionResponseType;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use tokio::sync::Mutex;

pub async fn start_command<GAME: DiscordDuelGame>(
    handler: &Handler<GAME>,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<()> {
    let options = command.data.options.as_slice();

    let program1_id = options
        .iter()
        .find_map(|option| {
            if option.name == "program1" {
                Some(option.value.as_ref()?.as_i64()? as usize)
            } else {
                None
            }
        })
        .ok_or(Error::msg("Please input a program1 id"))?;

    let program2_id = options
        .iter()
        .find_map(|option| {
            if option.name == "program2" {
                Some(option.value.as_ref()?.as_i64()? as usize)
            } else {
                None
            }
        })
        .ok_or(Error::msg("Please input a program1 id"))?;

    let file_path1 = PathBuf::from(format!("tmp/{}.wasm", program1_id));
    if !file_path1.try_exists()? {
        return Err(Error::msg("Program1 does not exists"));
    }

    let file_path2 = PathBuf::from(format!("tmp/{}.wasm", program2_id));
    if !file_path2.try_exists()? {
        return Err(Error::msg("Program2 does not exists"));
    }

    let config = GAME::Config::from_options(options);
    let game = GAME::new(config);

    // TODO: Use this option
    let _automatic = options.iter().any(|option| option.name == "automatic");

    let game_id = handler.number_game.fetch_add(1, Ordering::AcqRel);

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message
                        .content(format!("# Game {}\n{}", game_id, game.to_discord()))
                        .components(|c| {
                            c.create_action_row(|row| {
                                row.create_button(|button| {
                                    button
                                        .custom_id("play_button_id")
                                        .label("Play")
                                        .emoji(ReactionType::Unicode("▶️".to_string()))
                                        .style(ButtonStyle::Success)
                                });
                                row.create_button(|button| {
                                    button
                                        .custom_id("quit_button_id")
                                        .label("Quit")
                                        .emoji(ReactionType::Unicode("🛑".to_string()))
                                        .style(ButtonStyle::Danger)
                                })
                            })
                        })
                })
        })
        .await?;

    let mut games = handler.games.write().await;
    games.insert(
        game_id,
        Mutex::new(GameInstance {
            game,
            player_turn: PlayerTurn::Player1,
            file_player1: file_path1,
            file_player2: file_path2,
        }),
    );

    println!("Message sent");

    Ok(())
}

pub async fn add_command(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<()> {
    let options = command.data.options.as_slice();
    let attachment = get_attachment(options).ok_or(Error::msg("No valid attachment given"))?;

    if attachment.size > 100_000_000 {
        return Err(Error::msg(format!(
            "File too big ({}>100MB)",
            attachment.size
        )));
    }

    let content_type = attachment
        .content_type
        .as_ref()
        .ok_or(Error::msg("Attachment has no content type"))?;

    if content_type.as_str() != "application/wasm" {
        return Err(Error::msg(format!(
            "Attachment has bad content type: {}",
            content_type
        )));
    };

    let program_id = get_program_id(options).ok_or(Error::msg("No id given"))?;

    let file_path = PathBuf::from(format!("./tmp/{}.wasm", program_id));
    if file_path.exists() {
        return Err(Error::msg("Program id already exists"));
    }

    tokio::fs::write(file_path, attachment.download().await?).await?;

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content(format!("## Program added\n**id: {}**", program_id))
                })
        })
        .await?;

    Ok(())
}

fn get_attachment(options: &[CommandDataOption]) -> Option<&Attachment> {
    if let CommandDataOptionValue::Attachment(attachment) = options.get(0)?.resolved.as_ref()? {
        Some(attachment)
    } else {
        None
    }
}

fn get_program_id(options: &[CommandDataOption]) -> Option<usize> {
    if let CommandDataOptionValue::Integer(id) = options.get(1)?.resolved.as_ref()? {
        Some((*id).max(0) as usize)
    } else {
        None
    }
}
