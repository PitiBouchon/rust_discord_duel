use anyhow::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::InteractionResponseType;

pub fn create_list_command(
    command: &mut CreateApplicationCommand,
) -> &mut CreateApplicationCommand {
    command
        .name("list")
        .description("List the programs available")
}

pub async fn list_command(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<()> {
    let mut paths = tokio::fs::read_dir("./tmp/").await?;

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
                        .content(format!("## Program available:\n{}", files_list))
                        .ephemeral(true)
                })
        })
        .await
    {
        dbg!("Error quit: {}", why);
    }

    Ok(())
}
