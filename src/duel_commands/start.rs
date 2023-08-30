use std::path::PathBuf;
use std::sync::atomic::Ordering;
use anyhow::Error;
use duel_game::{DiscordConfig, DiscordDuelGame, PlayerTurn};
use serenity::client::Context;
use serenity::model::application::component::ButtonStyle;
use serenity::model::channel::ReactionType;
use serenity::model::prelude::application_command::{ApplicationCommandInteraction, CommandDataOptionValue};
use serenity::model::prelude::InteractionResponseType;
use tokio::sync::Mutex;
use crate::handler::{GameInstance, Handler};

pub async fn start_command<GAME: DiscordDuelGame>(
    handler: &Handler<GAME>,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> anyhow::Result<()> {
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
    let _automatic = options.iter().find_map(|option| {
        if option.name == "automatic" {
            match option.resolved.as_ref()? {
                CommandDataOptionValue::Boolean(b) => Some(*b),
                _ => None,
            }
        } else {
            None
        }
    })
        .unwrap_or(true);

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
