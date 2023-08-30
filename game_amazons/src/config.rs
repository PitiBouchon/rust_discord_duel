use duel_game::DiscordConfig;
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::command::CommandOptionType;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use std::fmt::{Display, Formatter};

pub struct AmazonsConfig {
    pub width: usize,
    pub height: usize,
    pub queens: usize,
}

impl AmazonsConfig {
    const WIDTH_NAME: &'static str = "width";
    const HEIGHT_NAME: &'static str = "height";
    const QUEENS_NAME: &'static str = "queens";
}

impl DiscordConfig for AmazonsConfig {
    fn create_command(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .create_option(|option| {
                option
                    .name(AmazonsConfig::WIDTH_NAME)
                    .description("Width of the grid (5-15)")
                    .required(false)
                    .kind(CommandOptionType::Integer)
            })
            .create_option(|option| {
                option
                    .name(AmazonsConfig::HEIGHT_NAME)
                    .description("Height of the grid (5-15)")
                    .required(false)
                    .kind(CommandOptionType::Integer)
            })
            .create_option(|option| {
                option
                    .name(AmazonsConfig::QUEENS_NAME)
                    .description("Number of queens (2-6)")
                    .required(false)
                    .kind(CommandOptionType::Integer)
            })
    }

    fn from_options(options: &[CommandDataOption]) -> Self {
        let mut width = 8;
        let mut height = 8;
        let mut queens = 2;
        for option in options.iter() {
            match option.name.as_str() {
                AmazonsConfig::WIDTH_NAME => {
                    if let Some(CommandDataOptionValue::Integer(width_desired)) = option.resolved {
                        width = width_desired.clamp(5, 15) as usize;
                    }
                }
                AmazonsConfig::HEIGHT_NAME => {
                    if let Some(CommandDataOptionValue::Integer(height_desired)) = option.resolved {
                        height = height_desired.clamp(5, 15) as usize;
                    }
                }
                AmazonsConfig::QUEENS_NAME => {
                    if let Some(CommandDataOptionValue::Integer(queens_desired)) = option.resolved {
                        queens = queens_desired.clamp(2, 6) as usize
                    }
                }
                _ => (),
            }
        }

        Self {
            width,
            height,
            queens,
        }
    }
}

impl Display for AmazonsConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Width: {} | Height: {} | Queens: {}",
            self.width, self.height, self.queens
        )
    }
}
