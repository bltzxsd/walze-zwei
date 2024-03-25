#![allow(unused_macros)]

#[rustfmt::skip]
pub enum EmbedColor {
    Error   = 0x00FF_3333,
    Ok      = 0x0072_A3C3,
}

impl From<EmbedColor> for poise::serenity_prelude::Colour {
    fn from(value: EmbedColor) -> Self {
        poise::serenity_prelude::Color::new(value as u32)
    }
}

#[macro_use]
pub(crate) mod discord {

    macro_rules! embed {
        ($ctx:expr, $title:expr, $desc:expr, $color:expr $(, $field_title:expr, $field_desc:expr)*) => {{

            let author_name = match $ctx.guild_id() {
                Some(id) => $ctx
                    .author()
                    .nick_in($ctx, id)
                    .await
                    .unwrap_or_else(|| $ctx.author().name.clone()),
                None => $ctx.author().name.clone(),
            };

            let author_icon_url = $ctx
                .author()
                .static_avatar_url()
                .unwrap_or_else(String::new);

            #[allow(unused_mut)] // TODO: look for a way to remove this allow.
            let mut embed = poise::serenity_prelude::CreateEmbed::new()
                .author(
                    poise::serenity_prelude::CreateEmbedAuthor::new(author_name).icon_url(author_icon_url),
                )
                .description($desc)
                .title($title)
                .color($color);

            $(
                embed = embed.field($field_title, $field_desc, false);
            )*

            embed
        }}
    }

    macro_rules! embed_error {
        ($ctx:expr, $title:expr, $desc:expr) => {{
            use $crate::utils::macros::discord::embed;
            embed!(
                $ctx,
                $title,
                $desc,
                $crate::utils::macros::EmbedColor::Error
            )
        }};
    }

    macro_rules! reply {
        ($ctx: expr, $title: expr, $desc:expr, $color:expr $(, $field_title:expr , $field_desc:expr)*) => {{
            use $crate::utils::macros::discord::embed;
            let embed = embed!($ctx, $title, $desc, $color $(, $field_title, $field_desc)*);
            poise::CreateReply::default().embed(embed)
        }};
    }

    macro_rules! reply_error {
        ($ctx:expr, $title:expr, $desc:expr) => {{
            use $crate::utils::macros::discord::embed_error;
            poise::CreateReply::default()
                .embed(embed_error!($ctx, $title, $desc))
                .ephemeral(true)
        }};
    }

    pub(crate) use embed;
    pub(crate) use embed_error;
    pub(crate) use reply;
    pub(crate) use reply_error;
}
