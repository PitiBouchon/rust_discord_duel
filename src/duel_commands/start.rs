use crate::duel_buttons::play::play_game_instance;
use crate::handler::{GameInstance, Handler};
use anyhow::Error;
use anyhow::Result;
use duel_game::{DiscordConfig, DiscordDuelGame, PlayerTurn};
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::http::Http;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::component::ButtonStyle;
use serenity::model::channel::ReactionType;
use serenity::model::prelude::application_command::{
    ApplicationCommandInteraction, CommandDataOptionValue,
};
use serenity::model::prelude::{ChannelId, InteractionResponseType, Message};
use std::ops::DerefMut;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::Mutex;

pub fn create_start_command<GAME: DiscordDuelGame>(
    command: &mut CreateApplicationCommand,
) -> &mut CreateApplicationCommand {
    let command = command
        .name("start")
        .description("Play a Duel Game with a .wasm file")
        .create_option(|option| {
            option
                .name("program1")
                .description("Program1")
                .required(true)
                .kind(CommandOptionType::Integer)
        })
        .create_option(|option| {
            option
                .name("program2")
                .description("Program2")
                .required(true)
                .kind(CommandOptionType::Integer)
        })
        .create_option(|option| {
            option
                .name("automatic")
                .description("Automatic")
                .required(false)
                .kind(CommandOptionType::Boolean)
        });
    GAME::Config::create_command(command)
}

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
    let displayed_config = format!("{}", config);
    let game = GAME::new(config);

    let automatic = options
        .iter()
        .find_map(|option| {
            if option.name == "automatic" {
                match option.resolved.as_ref()? {
                    CommandDataOptionValue::Boolean(b) => Some(*b),
                    _ => None,
                }
            } else {
                None
            }
        })
        .unwrap_or(false);

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content(format!(
                        "> # Game Info\n> {} **VS** {}\n> Options: {}",
                        program1_id, program2_id,
                        displayed_config
                    ))
                })
        })
        .await?;

    let mut message = command
        .create_followup_message(&ctx.http, |interaction| {
            interaction
                .content(format!("# Game\n{}", game.to_discord()))
                .components(|c| {
                    c.create_action_row(|row| {
                        if !automatic {
                            row.create_button(|button| {
                                button
                                    .custom_id("play_button_id")
                                    .label("Play")
                                    .emoji(ReactionType::Unicode("▶️".to_string()))
                                    .style(ButtonStyle::Success)
                            });
                        }
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
        .await?;

    let mut games = handler.games.write().await;
    games.insert(
        (message.channel_id, message.id),
        Mutex::new(GameInstance {
            game,
            player_turn: PlayerTurn::Player1,
            file_player1: file_path1,
            file_player2: file_path2,
        }),
    );
    drop(games);

    if automatic {
        let channel_id = command.channel_id;
        match loop_game(&ctx.http, handler, channel_id, &mut message).await {
            Ok(_) => (),
            Err(why) => {
                let games = handler.games.read().await;
                if let Some(game_lock) = games.get(&(channel_id, message.id)) {
                    let game_instance = game_lock.lock().await;
                    let winner = match game_instance.player_turn {
                        PlayerTurn::Player1 => 1,
                        PlayerTurn::Player2 => 2,
                    };
                    if let Some(info_message) = message.referenced_message.as_deref_mut() {
                        let info_message_content = info_message.content.clone();
                        info_message
                            .edit(&ctx.http, |interaction| {
                                interaction.content(format!(
                                    "{}\nProgram {} WIN because of error",
                                    info_message_content, winner
                                ))
                            })
                            .await?;
                    }
                }
                return Err(why);
            }
        }
    }

    Ok(())
}

async fn loop_game<GAME: DiscordDuelGame>(
    http: &Http,
    handler: &Handler<GAME>,
    channel_id: ChannelId,
    message: &mut Message,
) -> Result<()> {
    loop {
        let games = handler.games.read().await;
        match games.get(&(channel_id, message.id)) {
            None => break,
            Some(game_lock) => {
                let mut game_instance = game_lock.lock().await;
                let (end_state, discord_game_str) =
                    play_game_instance(game_instance.deref_mut()).await?;
                if end_state {
                    let winner = match game_instance.player_turn {
                        PlayerTurn::Player1 => 1,
                        PlayerTurn::Player2 => 2,
                    };
                    drop(game_instance); // Why do I need to drop it manually ?
                    drop(games);
                    let mut games = handler.games.write().await;
                    let _ = games.remove(&(channel_id, message.id));
                    if let Some(info_message) = message.referenced_message.as_deref_mut() {
                        let info_message_content = info_message.content.clone();
                        info_message
                            .edit(http, |interaction| {
                                interaction.content(format!(
                                    "{}\nProgram {} WIN",
                                    info_message_content, winner
                                ))
                            })
                            .await?;
                    }
                    break;
                } else {
                    message
                        .edit(http, |interaction| {
                            interaction.content(format!("# Game\n{}", discord_game_str))
                        })
                        .await?;
                }
            }
        }
        drop(games);
        tokio::time::sleep(Duration::from_millis(300)).await;
    }

    Ok(())
}
