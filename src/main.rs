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

use commands::context_cmd;
use commands::eval;
use commands::tz;
use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use serenity::UserId;
use tokio::{fs::OpenOptions, io::AsyncReadExt};
use tracing::{debug, error, info};
use walzecore::db::Users;

use crate::{
    commands::alias,
    error::Result,
    models::{Context, Data},
    utils::macros::discord::reply_error,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_ansi(true)
        .with_max_level(tracing::Level::INFO)
        .init();

    if let Err(e) = run().await {
        error!("{:#?}", e);
    }
}

async fn run() -> Result<()> {
    dotenv().ok();

    let users = load_users_from_file().await?;
    let data = Data::new(users);

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
                debug!("commands registered globally");
                Ok(data)
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
                error!("failed to dispatch error response: {e:#?}");
            }
        }
        _ => {}
    }
}

// Load the users data from JSON file
async fn load_users_from_file() -> Result<Users<UserId>> {
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open("users.json")
        .await?;

    let mut json = String::new();
    file.read_to_string(&mut json).await?;

    if json.is_empty() {
        json.push_str("{}");
    }

    let users = Users::new(&json)?;
    Ok(users)
}
