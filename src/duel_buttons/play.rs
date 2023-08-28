use crate::duel_buttons::parse_first_line_game_id;
use crate::handler::Handler;
use anyhow::{Error, Result};
use duel_game::{DiscordDuelGame, PlayerTurn};
use serenity::model::prelude::message_component::MessageComponentInteraction;
use serenity::model::prelude::InteractionResponseType;
use serenity::prelude::Context;
use std::io::IoSlice;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use wasi_common::pipe::{ReadPipe, WritePipe};
use wasi_common::WasiFile;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;

pub async fn play_button<GAME: DiscordDuelGame>(
    handler: &Handler<GAME>,
    ctx: &Context,
    command: &MessageComponentInteraction,
) -> Result<()> {
    let game_id = parse_first_line_game_id(command)?;
    let games = handler.games.read().await;
    let game_lock = games
        .get(&game_id)
        .ok_or(Error::msg(format!("Game {} does not exists", game_id)))?;
    let mut game_data = game_lock.lock().await;
    let game_instance = game_data.deref_mut();
    let file_path = match game_instance.player_turn {
        PlayerTurn::Player1 => game_instance.file_player1.clone(),
        PlayerTurn::Player2 => game_instance.file_player2.clone(),
    };
    let input = run_file(
        &game_instance.game,
        file_path.as_path(),
        &game_instance.player_turn,
    )
    .await?;
    let end_state = match game_instance.game.play(input, game_instance.player_turn) {
        Ok(state) => state,
        Err(why) => return Err(Error::msg(format!("Error playing: {}", why))),
    };
    game_instance.player_turn = game_instance.player_turn.next();
    if end_state {
        let discord_game_str = game_instance.game.to_discord();
        drop(game_data); // Why do I need to drop it manually ?
        drop(games);
        let mut games = handler.games.write().await;
        let _ = games.remove(&game_id);
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|message| {
                        message
                            .content(format!("# Game ENDED\n{}", discord_game_str))
                            .components(|c| c)
                    })
            })
            .await?
    } else {
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|message| {
                        message.content(format!(
                            "# Game {}\n{}",
                            game_id,
                            game_instance.game.to_discord()
                        ))
                    })
            })
            .await?
    }

    Ok(())
}

async fn run_file<GAME: DiscordDuelGame>(
    game: &GAME,
    file_path: &Path,
    n: &PlayerTurn,
) -> Result<GAME::Input> {
    let console_str = match n {
        PlayerTurn::Player1 => game.to_console_player1(),
        PlayerTurn::Player2 => game.to_console_player2(),
    };
    let file_path = file_path.to_path_buf();
    let handle = tokio::task::spawn(run_wasm(console_str, file_path));
    let output_str = tokio::time::timeout(Duration::from_secs(2), handle).await???;
    match GAME::Input::from_str(output_str.as_str()) {
        Ok(game_input) => Ok(game_input),
        Err(_) => Err(Error::msg(format!("Error parsing game input"))),
    }
}

async fn run_wasm(grid_console_string: String, file_path: PathBuf) -> Result<String> {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

    let stdin = ReadPipe::from(grid_console_string);
    let stdout = WritePipe::new_in_memory();

    let wasi = WasiCtxBuilder::new()
        .stdin(Box::new(stdin))
        .stdout(Box::new(stdout.clone()))
        .build();

    let mut store = Store::new(&engine, wasi);
    let module = Module::from_file(&engine, file_path)?;
    linker.module(&mut store, "", &module)?;

    linker
        .get_default(&mut store, "")?
        .typed::<(), ()>(&store)?
        .call(&mut store, ())?;

    drop(store);
    match stdout.try_into_inner() {
        Ok(res) => {
            let bytes_res = res.into_inner();
            let str_res = String::from_utf8_lossy(bytes_res.as_slice()).to_string();
            println!("Result: {:?}", str_res);
            Ok(str_res)
        }
        Err(_) => Err(Error::msg("Error getting stdout result")),
    }
}
