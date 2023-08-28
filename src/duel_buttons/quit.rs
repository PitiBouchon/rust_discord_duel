use crate::duel_buttons::parse_first_line_game_id;
use crate::handler::Handler;
use anyhow::Result;
use duel_game::DiscordDuelGame;
use serenity::model::prelude::message_component::MessageComponentInteraction;
use serenity::model::prelude::InteractionResponseType;
use serenity::prelude::Context;

pub async fn quit_button<GAME: DiscordDuelGame>(
    handler: &Handler<GAME>,
    ctx: &Context,
    command: &MessageComponentInteraction,
) -> Result<()> {
    let game_id = parse_first_line_game_id(command)?;

    let mut games = handler.games.write().await;
    let _ = games.remove(&game_id);

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|message| message.components(|c| c))
        })
        .await?;

    Ok(())
}
