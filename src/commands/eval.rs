use crate::{
    utils::{
        self,
        macros::{discord::embed, EmbedColor},
    },
    Context, Result,
};

use caith::Roller;

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
