use crate::{
    error::Result,
    models::Context,
    utils::macros::{
        discord::{embed, reply},
        EmbedColor,
    },
};
use futures_util::{future, stream, Stream, StreamExt};
use poise::serenity_prelude::CreateEmbedFooter;

#[allow(clippy::unused_async)]
#[poise::command(
    slash_command,
    subcommands("create_alias", "delete_alias", "dump_alias")
)]
pub async fn alias(_: Context<'_>) -> Result<()> {
    Ok(())
}

/// delete given alias if it exists in current namespace
#[poise::command(slash_command, rename = "remove")]
pub async fn delete_alias(ctx: Context<'_>, alias: String) -> Result<()> {
    let mut user = ctx.data().lock().await;
    let user = user.get_or_create(ctx.author().id);

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

/// mutate (update or create) aliases inside the current namespace.
#[poise::command(slash_command, rename = "mutate")]
pub async fn create_alias(ctx: Context<'_>, var: String, be: String) -> Result<()> {
    let mut user = ctx.data().lock().await;
    let user = user.get_or_create(ctx.author().id);
    let title = if user.aliases()?.contains(&(&var, &be)) {
        "Updated Alias"
    } else {
        "Added Alias"
    };

    user.alias_mut("$".to_owned() + &var, be.clone())?;
    let namespace = user.namespace();

    let reply = reply!(
        ctx,
        title,
        format!("{var} -> {be} in {namespace}"),
        EmbedColor::Ok
    );
    ctx.send(reply.ephemeral(true)).await?;
    Ok(())
}

/// returns all the aliases stored in the current namespace
#[poise::command(slash_command, rename = "dump")]
pub async fn dump_alias(ctx: Context<'_>) -> Result<()> {
    let mut user = ctx.data().lock().await;
    let user = user.get_or_create(ctx.author().id);
    let aliases = user.aliases()?;
    if aliases.is_empty() {
        let reply = reply!(
            ctx,
            "Current Aliases",
            "No aliases set!\nSet some using the `/alias mutate` command",
            EmbedColor::Ok
        )
        .ephemeral(true);
        ctx.send(reply).await?;
        return Ok(());
    }

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

#[allow(clippy::unused_async)]
#[poise::command(
    slash_command,
    subcommands(
        "namespace_switch",
        "namespace_new",
        "namespace_dump",
        "namespace_delete"
    )
)]
pub async fn namespace(_: Context<'_>) -> Result<()> {
    Ok(())
}

/// swtich the current namespace to given namespace
#[poise::command(slash_command, rename = "switch")]
pub async fn namespace_switch(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_namespace"] namespace: String,
) -> Result<()> {
    let mut user = ctx.data().lock().await;
    let user = user.get_or_create(ctx.author().id);

    if !user.namespaces().contains(&namespace) {
        return Err(walzecore::db::Error::NamespaceNotFound(namespace).into());
    }

    let desc = format!(
        "Switched namespaces: {} -> {}",
        user.namespace(),
        &namespace
    );
    user.namespace_mut(namespace);
    let reply = reply!(ctx, "Switched namespace", desc, EmbedColor::Ok).ephemeral(true);
    ctx.send(reply).await?;
    Ok(())
}

/// create a new namespace
#[poise::command(slash_command, rename = "create")]
pub async fn namespace_new(ctx: Context<'_>, namespace: String) -> Result<()> {
    let mut user = ctx.data().lock().await;
    let user = user.get_or_create(ctx.author().id);

    if user.namespaces().contains(&namespace) {
        return Err(
            walzecore::db::Error::Simple("cannot rewrite over an existing namespace").into(),
        );
    }

    let desc = format!("added namespace {}", &namespace);
    user.add_namespace(namespace);
    let reply = reply!(ctx, "Added namespace", desc, EmbedColor::Ok).ephemeral(true);
    ctx.send(reply).await?;
    Ok(())
}

/// return all stored namespaces
#[poise::command(slash_command, rename = "dump")]
pub async fn namespace_dump(ctx: Context<'_>) -> Result<()> {
    let mut user = ctx.data().lock().await;
    let user = user.get_or_create(ctx.author().id);
    let namespaces = user.namespaces().join("\n");

    let reply = reply!(
        ctx,
        "Stored Namespaces",
        format!("```\n{namespaces}\n```"),
        EmbedColor::Ok
    )
    .ephemeral(true);
    ctx.send(reply).await?;
    Ok(())
}

/// delete a given namespace if it exists
#[poise::command(slash_command, rename = "delete")]
pub async fn namespace_delete(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_namespace"] namespace: String,
) -> Result<()> {
    let mut user = ctx.data().lock().await;
    let user = user.get_or_create(ctx.author().id);
    if namespace.as_str() == "default" {
        return Err(walzecore::db::Error::Simple("cannot drop default namespace").into());
    }
    let (popped_ns, aliases) = user.remove_namespace(&namespace)?;
    let aliases = aliases
        .into_iter()
        .fold(String::from("Removed Aliases: "), |mut acc, (k, v)| {
            acc.push_str(format!("\t|-> {k} -> {v}\n").as_str());
            acc
        });
    let reply = reply!(
        ctx,
        "Removed Namespace: ".to_owned() + &popped_ns,
        aliases,
        EmbedColor::Ok
    )
    .ephemeral(true);
    ctx.send(reply).await?;
    Ok(())
}

async fn autocomplete_namespace<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let partial = partial.to_lowercase();
    let namespaces = ctx
        .data()
        .lock()
        .await
        .get_or_create(ctx.author().id)
        .namespaces();

    stream::iter(namespaces).filter(move |ns| future::ready(ns.to_lowercase().contains(&partial)))
}
