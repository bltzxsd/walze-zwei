#![warn(clippy::style)]
#![warn(clippy::complexity)]
#![warn(clippy::perf)]
#![warn(clippy::pedantic)]
#![deny(clippy::correctness)]
#![allow(clippy::similar_names)]

mod commands;
mod error;
mod models;
mod utils;

use std::sync::Arc;

use commands::context_cmd;
use commands::eval;
use commands::tz;
use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use tokio::sync::Mutex;
use tokio::{fs::OpenOptions, io::AsyncReadExt};
use tracing::{debug, error, info};
use walzecore::db::Users;

use models::{Context, Data, Inner};
use crate::{
    commands::alias,
    error::Result,
    utils::macros::discord::reply_error,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .pretty()
        .with_ansi(true)
        .with_max_level(tracing::Level::INFO)
        .init();

    if let Err(e) = setup_client().await {
        error!("{:#?}", e);
    }
}

async fn setup_client() -> Result<()> {
    dotenv().expect("no .env file found");

    // Load the existing users data from the JSON file
    let users: Users<serenity::UserId> = load_users_from_file().await?;

    // Create a new Inner struct with the loaded users data
    let db = Arc::new(Mutex::new(Inner::new(users)));

    let token = std::env::var("DISCORD_API")?;
    let intents = serenity::GatewayIntents::non_privileged();
    let commands = vec![
        eval::eval(),
        alias::alias(),
        alias::namespace(),
        context_cmd::help(),
        context_cmd::echo(),
        tz::tzcalc(),
    ];

    let options = poise::FrameworkOptions {
        commands,
        on_error: |err| Box::pin(on_error(err)),
        pre_command: |ctx| {
            Box::pin(async move {
                info!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        post_command: |ctx| {
            Box::pin(async move {
                info!("Executed command {}", ctx.command().qualified_name);
            })
        },
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                debug!(
                    "Got an event in event handler: {:?}",
                    event.snake_case_name(),
                );
                Ok(())
            })
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .options(options)
        .setup(|ctx, _, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                info!("commands registered globally");
                Ok(Data::new(db.clone()))
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to handle ctrl-c signal");
        shard_manager.shutdown_all().await;
        info!("shutting down");
    });

    client.start().await?;
    Ok(())
}

async fn on_error(err: poise::FrameworkError<'_, Data, crate::error::Error>) {
    use poise::FrameworkError::{Command, Setup};

    match err {
        Setup { error, .. } => panic!("failed to start bot: {error:#?}"),
        Command { error, ctx, .. } => {
            let reply = reply_error!(ctx, "Error", error.to_string());
            if let Err(e) = ctx.send(reply).await {
                panic!("failed to dispatch error response: {e:#?}");
            }
        }
        _ => {}
    }
}

// Load the users data from the JSON file
async fn load_users_from_file() -> Result<Users<serenity::UserId>> {
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open("users.json")
        .await?;

    let mut json = String::new();
    file.read_to_string(&mut json).await?;

    // If the file is empty, initialize with an empty JSON object
    if json.is_empty() {
        json += "{}";
    }

    let users: Users<serenity::UserId> = Users::new(&json)?;
    Ok(users)
}
