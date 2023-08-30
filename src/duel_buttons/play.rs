use crate::handler::{GameInstance, Handler};
use anyhow::{Error, Result};
use duel_game::{DiscordDuelGame, PlayerTurn};
use serenity::model::prelude::message_component::MessageComponentInteraction;
use serenity::prelude::Context;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
use wasi_common::pipe::{ReadPipe, WritePipe};
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;

pub async fn play_button<GAME: DiscordDuelGame>(
    handler: &Handler<GAME>,
    ctx: &Context,
    command: &MessageComponentInteraction,
) -> Result<()> {
    let message_id = command.message.id;
    let channel_id = command.channel_id;

    let games = handler.games.read().await;
    let game_lock = games
        .get(&(channel_id, message_id))
        .ok_or(Error::msg(format!(
            "MessageId {} does not exists",
            message_id
        )))?;
    let mut game_instance = game_lock.lock().await;
    let (end_state, discord_game_str) = play_game_instance(game_instance.deref_mut()).await?;

    let mut message = channel_id.message(&ctx.http, message_id).await?;
    if end_state {
        let winner = match game_instance.player_turn {
            PlayerTurn::Player1 => 1,
            PlayerTurn::Player2 => 2,
        };
        drop(game_instance); // Why do I need to drop it manually ?
        drop(games);
        let mut games = handler.games.write().await;
        let _ = games.remove(&(channel_id, message_id));
        if let Some(mut info_message) = message.referenced_message {
            let info_message_content = info_message.content.clone();
            info_message
                .edit(&ctx.http, |interaction| {
                    interaction.content(format!("{}\nProgram {} WIN", info_message_content, winner))
                })
                .await?;
        }
    } else {
        message
            .edit(&ctx.http, |message| {
                message.content(format!("# Game\n{}", discord_game_str))
            })
            .await?;
    }

    Ok(())
}

pub async fn play_game_instance<GAME: DiscordDuelGame>(
    game_instance: &mut GameInstance<GAME>,
) -> Result<(bool, String)> {
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

    Ok((end_state, game_instance.game.to_discord()))
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
    let sleep = sleep(Duration::from_secs(3));
    tokio::pin!(sleep);

    tokio::select! {
        _ = &mut sleep, if !sleep.is_elapsed() => {
            Err(Error::msg("Program timed out (>3s)"))
        }
        res = run_wasm(console_str, file_path) => {
            let output_str = res?;
            match GAME::Input::from_str(output_str.as_str()) {
                Ok(game_input) => Ok(game_input),
                Err(_) => Err(Error::msg(format!("Error parsing game input: {}", output_str))),
            }
        }
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
            Ok(str_res)
        }
        Err(_) => Err(Error::msg("Error getting stdout result")),
    }
}
