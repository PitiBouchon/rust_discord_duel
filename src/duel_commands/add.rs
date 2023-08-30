use std::path::PathBuf;
use anyhow::Error;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::channel::Attachment;
use serenity::model::prelude::application_command::{ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::InteractionResponseType;

pub fn make_add_command(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
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
}

pub async fn add_command(ctx: &Context, command: &ApplicationCommandInteraction) -> anyhow::Result<()> {
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
