use crate::handler::Handler;
use anyhow::Result;
use duel_game::DiscordDuelGame;
use serenity::model::prelude::message_component::MessageComponentInteraction;
use serenity::prelude::Context;

pub async fn quit_button<GAME: DiscordDuelGame>(
    handler: &Handler<GAME>,
    ctx: &Context,
    command: &MessageComponentInteraction,
) -> Result<()> {
    let message_id = command.message.id;
    let channel_id = command.channel_id;

    let mut games = handler.games.write().await;
    let _ = games.remove(&(channel_id, message_id));

    let mut message = channel_id.message(&ctx.http, message_id).await?;
    message
        .edit(&ctx.http, |interaction| interaction.components(|c| c))
        .await?;

    if let Some(mut info_message) = message.referenced_message {
        let info_message_content = info_message.content.clone();
        info_message
            .edit(&ctx.http, |interaction| {
                interaction.content(format!("{}\nGAME QUIT", info_message_content))
            })
            .await?;
    }

    Ok(())
}
