mod duel_buttons;
mod duel_commands;
mod handler;

use handler::Handler;
use konst::primitive::parse_u64;
use konst::unwrap_ctx;
use serenity::prelude::GatewayIntents;
use serenity::Client;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use tokio::fs::{create_dir_all, remove_dir_all};
use tokio::sync::RwLock;

const TOKEN: &str = include_str!("../res/token.txt");
const APPLICATION_ID: u64 = unwrap_ctx!(parse_u64(include_str!("../res/application_id.txt")));

#[tokio::main]
async fn main() {
    // let _ = remove_dir_all("./tmp/").await;
    create_dir_all("./tmp/").await.unwrap();

    let intents = GatewayIntents::empty();

    let mut client = Client::builder(TOKEN, intents)
        .event_handler(Handler::<game_amazons::AmazonsGame> {
            number_game: AtomicUsize::new(0),
            games: RwLock::new(HashMap::with_capacity(10)),
        })
        .application_id(APPLICATION_ID)
        .await
        .expect("Error creating the client");

    if let Err(why) = client.start().await {
        dbg!("Error: {}", why);
    }
}
