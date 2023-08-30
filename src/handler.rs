use crate::duel_buttons::play::play_button;
use crate::duel_buttons::quit::quit_button;
use crate::duel_commands::add::{add_command, create_add_command};
use crate::duel_commands::list::{create_list_command, list_command};
use crate::duel_commands::start::{
    create_start_command, start_command, PLAY_BUTTON_ID, QUIT_BUTTON_ID,
};
use duel_game::{DiscordDuelGame, PlayerTurn};
use serenity::async_trait;
use serenity::http::Http;
use serenity::model::id::ChannelId;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::message_component::MessageComponentInteraction;
use serenity::model::prelude::{GuildId, Interaction, MessageId, Ready};
use serenity::prelude::{Context, EventHandler};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::{Mutex, RwLock};

pub struct GameInstance<GAME: DiscordDuelGame> {
    pub game: GAME,
    pub player_turn: PlayerTurn,
    pub file_player1: PathBuf,
    pub file_player2: PathBuf,
}

pub struct Handler<GAME: DiscordDuelGame> {
    // pub number_game: AtomicUsize,
    pub games: RwLock<HashMap<(ChannelId, MessageId), Mutex<GameInstance<GAME>>>>,
}

#[async_trait]
impl<GAME: DiscordDuelGame> EventHandler for Handler<GAME> {
    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        println!("Discord Bot \"{}\" is connected", data_about_bot.user.name);

        for guild in data_about_bot.guilds {
            println!("In guild : {}", guild.id);
            let guild_id = guild.id;

            let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
                commands
                    .create_application_command(|command| create_add_command(command))
                    .create_application_command(|command| create_list_command(command))
                    .create_application_command(|command| create_start_command::<GAME>(command))
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
                }
                "add" => {
                    if let Err(error) = add_command(&ctx, &command).await {
                        send_error_application_command(&ctx.http, command, error).await;
                    }
                }
                _ => unreachable!(),
            },
            Interaction::MessageComponent(command) => match command.data.custom_id.as_str() {
                PLAY_BUTTON_ID => {
                    if let Err(error) = play_button(self, &ctx, &command).await {
                        send_error_message_component(&ctx.http, command, error).await;
                    }
                }
                QUIT_BUTTON_ID => {
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

async fn send_error_message_component(
    http: impl AsRef<Http>,
    command: MessageComponentInteraction,
    error: anyhow::Error,
) {
    if let Err(why) = command
        .create_followup_message(http, |interaction| {
            interaction.ephemeral(true).content(error)
        })
        .await
    {
        dbg!("Error: {}", why);
    }
}

async fn send_error_application_command(
    http: impl AsRef<Http>,
    command: ApplicationCommandInteraction,
    error: anyhow::Error,
) {
    if let Err(why) = command
        .create_followup_message(http, |interaction| {
            interaction.ephemeral(true).content(error)
        })
        .await
    {
        dbg!("Error: {}", why);
    }
}
