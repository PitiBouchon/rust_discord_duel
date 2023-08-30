use crate::duel_buttons::play::play_button;
use crate::duel_buttons::quit::quit_button;
use crate::duel_commands::add::{add_command, make_add_command};
use crate::duel_commands::start::start_command;
use duel_game::{DiscordConfig, DiscordDuelGame, PlayerTurn};
use serenity::async_trait;
use serenity::model::application::command::CommandOptionType;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use serenity::http::Http;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::message_component::MessageComponentInteraction;
use serenity::model::prelude::{GuildId, Interaction, InteractionResponseType, Ready};
use serenity::prelude::{Context, EventHandler};
use tokio::sync::{Mutex, RwLock};
use crate::duel_commands::list::{list_command, make_list_command};

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
                        make_add_command(command)
                    })
                    .create_application_command(|command| {
                        make_list_command(command)
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
                        send_error_application_command(&ctx.http, command, error).await;
                    }
                }
                "list" => {
                    if let Err(error) = list_command(&ctx, &command).await {
                        send_error_application_command(&ctx.http, command, error).await;
                    }
                },
                "add" => {
                    if let Err(error) = add_command(&ctx, &command).await {
                        send_error_application_command(&ctx.http, command, error).await;
                    }
                }
                _ => unreachable!(),
            },
            Interaction::MessageComponent(command) => match command.data.custom_id.as_str() {
                "play_button_id" => {
                    if let Err(error) = play_button(self, &ctx, &command).await {
                        send_error_message_component(&ctx.http, command, error).await;
                    }
                }
                "quit_button_id" => {
                    if let Err(error) = quit_button(self, &ctx, &command).await {
                        send_error_message_component(&ctx.http, command, error).await;
                    }
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

async fn send_error_message_component(http: impl AsRef<Http>, command: MessageComponentInteraction, error: anyhow::Error) {
    if let Err(why) = command
        .create_interaction_response(http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content(error).ephemeral(true)
                })
        })
        .await
    {
        dbg!("Error: {}", why);
    }
}

async fn send_error_application_command(http: impl AsRef<Http>, command: ApplicationCommandInteraction, error: anyhow::Error) {
    if let Err(why) = command
        .create_interaction_response(http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content(error).ephemeral(true)
                })
        })
        .await
    {
        dbg!("Error: {}", why);
    }
}

