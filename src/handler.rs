use crate::duel_buttons::play::play_button;
use crate::duel_buttons::quit::quit_button;
use crate::duel_command::{add_command, start_command};
use duel_game::{DiscordConfig, DiscordDuelGame, PlayerTurn};
use serenity::async_trait;
use serenity::model::application::command::CommandOptionType;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use tokio::sync::{Mutex, RwLock};

pub struct GameInstance<GAME: DiscordDuelGame> {
    pub game: GAME,
    pub player_turn: PlayerTurn,
    pub file_player1: PathBuf,
    pub file_player2: PathBuf,
}

pub struct Handler<GAME: DiscordDuelGame> {
    pub number_game: AtomicUsize,
    pub games: RwLock<HashMap<usize, Mutex<GameInstance<GAME>>>>,
}

#[async_trait]
impl<GAME: DiscordDuelGame> EventHandler for Handler<GAME> {
    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        println!("{} is connected !", data_about_bot.user.name);

        for guild in data_about_bot.guilds {
            println!("In guild : {} | {}", guild.id, guild.unavailable);
            let guild_id = guild.id;

            let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
                commands
                    .create_application_command(|command| {
                        command
                            .name("add")
                            .description("Add a .wasm file to the list of the programs")
                            .create_option(|option| {
                                option
                                    .name("attachment")
                                    .description("A wasm file")
                                    .required(true)
                                    .kind(CommandOptionType::Attachment)
                            })
                            .create_option(|option| {
                                option
                                    .name("id")
                                    .description("Id of the program")
                                    .required(true)
                                    .kind(CommandOptionType::Integer)
                            })
                    })
                    .create_application_command(|command| {
                        command
                            .name("list")
                            .description("List the programs available")
                    })
                    .create_application_command(|command| {
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
                    })
            })
            .await;

            if let Err(why) = commands {
                dbg!("Failed creating commands: {}", why);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => match command.data.name.as_str() {
                "start" => {
                    if let Err(error) = start_command::<GAME>(self, &ctx, &command).await {
                        if let Err(why) = command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content(error).ephemeral(true)
                                    })
                            })
                            .await
                        {
                            dbg!("Error start: {}", why);
                        }
                    }
                }
                "list" => match tokio::fs::read_dir("./tmp/").await {
                    Ok(mut paths) => {
                        let mut files: Vec<String> = Vec::new();
                        while let Ok(Some(path)) = paths.next_entry().await {
                            if let Ok(filename) = path.file_name().into_string() {
                                files.push(format!("- `{}`", filename));
                            }
                        }
                        let files_list = files.join("\n");
                        if let Err(why) = command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message
                                            .content(format!(
                                                "## Program available:\n{}",
                                                files_list
                                            ))
                                            .ephemeral(true)
                                    })
                            })
                            .await
                        {
                            dbg!("Error quit: {}", why);
                        }
                    }
                    Err(why) => {
                        if let Err(why) = command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message
                                            .content(format!(
                                                "Can't get the list of programs: {}",
                                                why
                                            ))
                                            .ephemeral(true)
                                    })
                            })
                            .await
                        {
                            dbg!("Error quit: {}", why);
                        }
                    }
                },
                "add" => {
                    if let Err(why) = add_command(&ctx, &command).await {
                        if let Err(why) = command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content(format!("Error: {}", why)).ephemeral(true)
                                    })
                            })
                            .await
                        {
                            dbg!("Error quit: {}", why);
                        }
                    }
                }
                _ => unreachable!(),
            },
            Interaction::MessageComponent(command) => match command.data.custom_id.as_str() {
                "play_button_id" => {
                    if let Err(error) = play_button(self, &ctx, &command).await {
                        if let Err(why) = command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content(&error).ephemeral(true)
                                    })
                            })
                            .await
                        {
                            dbg!("Error play: {} | {}", why, error);
                        }
                    }
                }
                "quit_button_id" => {
                    if let Err(error) = quit_button(self, &ctx, &command).await {
                        if let Err(why) = command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content(error).ephemeral(true)
                                    })
                            })
                            .await
                        {
                            dbg!("Error play: {}", why);
                        }
                    }
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}
