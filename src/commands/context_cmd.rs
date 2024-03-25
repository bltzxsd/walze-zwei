use crate::Context;
use crate::Result;

use poise::serenity_prelude as serenity;

/// repeats the selected message
#[poise::command(context_menu_command = "Echo", slash_command)]
pub async fn echo(ctx: Context<'_>, msg: serenity::Message) -> Result<()> {
    ctx.say(&msg.content).await?;
    Ok(())
}

/// show help menu
#[poise::command(slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<()> {
    let config = poise::builtins::HelpConfiguration::default();
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}
