use crate::utils::macros::discord::embed;
use crate::utils::macros::discord::reply;

use crate::utils::macros::EmbedColor;
use crate::Context;
use crate::Result;

use caith::Roller;

use poise::serenity_prelude::CreateEmbedFooter;
use walzecore::db::database;

/// evaluate a dice string and return the result
#[poise::command(slash_command)]
pub async fn eval(
    ctx: Context<'_>,
    #[description = "Evaluate this dice expression"] expr: String,
    #[description = "Show the dice roll in chat"] show: Option<bool>,
) -> Result<()> {
    let data = ctx.data();

    let mut user = data.lock().await;
    let aliases = user.get_or_create(ctx.author().id).aliases()?;

    let resolved_expr = aliases
        .iter()
        .fold(expr, |acc, (alias, value)| acc.replace(alias, value));

    let die = utils::split_dice(&resolved_expr);
    let mut embeds = Vec::with_capacity(die.len());

    for roll in die {
        let roller = Roller::new(roll)?;
        let result = roller
            .roll()
            .map_err(|e| format!("error while parsing input: {roll}\n```\n{e}\n```"))?;

        let result = utils::normalize_dice_expr(result.to_string().as_ref());
        embeds.push(embed!(ctx, roll, result, EmbedColor::Ok));
    }

    let reply = embeds
        .into_iter()
        .fold(poise::CreateReply::default(), |reply, embed| {
            reply.embed(embed)
        })
        .ephemeral(show.unwrap_or(false));

    ctx.send(reply).await?;
    Ok(())
}

#[allow(clippy::unused_async)]
#[poise::command(
    slash_command,
    subcommands("create_alias", "delete_alias", "dump_alias")
)]
pub async fn alias(_: Context<'_>) -> Result<()> {
    Ok(())
}

#[poise::command(slash_command, rename = "remove")]
pub async fn delete_alias(ctx: Context<'_>, alias: String) -> Result<()> {
    let mut user = ctx.data().lock().await;
    let author = ctx.author().id;
    let user = user.entry(author).or_insert(database::User::new());

    let removed_alias = user.remove_alias(format!("${alias}"))?;
    let footer = CreateEmbedFooter::new(format!("namespace: {}", user.namespace()));
    let reply = embed!(
        ctx,
        "Removed alias",
        format!("removed\n{} -> {}", alias, removed_alias),
        EmbedColor::Ok
    )
    .footer(footer);
    ctx.send(poise::CreateReply::default().embed(reply).ephemeral(true))
        .await?;

    Ok(())
}

/// The `/alias mutate` command can mutate aliases inside the current namespace.
#[poise::command(slash_command, rename = "mutate")]
pub async fn create_alias(ctx: Context<'_>, var: String, be: String) -> Result<()> {
    let mut user = ctx.data().lock().await;
    let author = ctx.author().id;
    let user = user.entry(author).or_insert(database::User::new());
    user.alias_mut("$".to_owned() + &var, be.clone())?;
    let namespace = user.namespace();

    let reply = reply!(
        ctx,
        "Added Alias",
        format!("{var} -> {be} in {namespace}"),
        EmbedColor::Ok
    );
    ctx.send(reply.ephemeral(true)).await?;
    Ok(())
}

// `/alias dump` returns all the aliases stored in the current namespace
#[poise::command(slash_command, rename = "dump")]
pub async fn dump_alias(ctx: Context<'_>) -> Result<()> {
    let mut user = ctx.data().lock().await;
    let user = user.entry(ctx.author().id).or_insert(database::User::new());
    let aliases = user.aliases()?;

    let mut desc = String::with_capacity(aliases.len() * 4usize + 8usize);
    desc.push_str("```\n");
    for (k, v) in aliases {
        desc.push_str(k);
        desc.push_str(" -> ");
        desc.push_str(v);
        desc.push('\n');
    }
    desc.push_str("\n```");
    let reply = embed!(ctx, "Current Aliases", desc, EmbedColor::Ok);
    let footer = CreateEmbedFooter::new("namespace: ".to_owned() + user.namespace());
    let embed = reply.footer(footer);
    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    Ok(())
}

fn normalize_dice_expr(s: &str) -> String {
    s.replace(['*', '`'], "")
}

fn split_dice_string(dice_str: &str) -> Vec<&str> {
    dice_str
        .trim()
        .split(',')
        .filter(|&s| !s.is_empty())
        .map(str::trim)
        .collect()
}
