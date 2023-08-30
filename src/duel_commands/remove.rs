// use serenity::builder::CreateApplicationCommand;
// use serenity::client::Context;
// use serenity::model::prelude::application_command::ApplicationCommandInteraction;
// use serenity::model::prelude::InteractionResponseType;
//
// TODO: Do the remove command to remove a program (file)
// pub fn create_remove_command(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
//     command
//         .name("remove")
//         .description("Remove a program")
// }
//
// pub async fn remove_command(
//     ctx: &Context,
//     command: &ApplicationCommandInteraction,
// ) -> anyhow::Result<()> {
//     let mut paths = tokio::fs::read_dir("./tmp/").await?;
//
//     let mut files: Vec<String> = Vec::new();
//     while let Ok(Some(path)) = paths.next_entry().await {
//         if let Ok(filename) = path.file_name().into_string() {
//             files.push(format!("- `{}`", filename));
//         }
//     }
//     let files_list = files.join("\n");
//     if let Err(why) = command
//         .create_interaction_response(&ctx.http, |response| {
//             response
//                 .kind(InteractionResponseType::ChannelMessageWithSource)
//                 .interaction_response_data(|message| {
//                     message
//                         .content(format!(
//                             "## Program available:\n{}",
//                             files_list
//                         ))
//                         .ephemeral(true)
//                 })
//         })
//         .await
//     {
//         dbg!("Error quit: {}", why);
//     }
//
//     Ok(())
// }
