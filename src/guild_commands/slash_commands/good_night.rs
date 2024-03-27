use crate::image_cache::ImageCache;
use crate::{Error, Result, COLLEGE_KINGS_GUILD_ID};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateCommand, CreateEmbed, EditAttachments,
    EditInteractionResponse, GuildId,
};

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
    interaction.defer(&ctx).await?;

    let data = ctx.data.read().await;
    let image_cache = data
        .get::<ImageCache>()
        .ok_or_else(|| Error::DataNotFound)?;

    let entries = &image_cache.good_night_images;

    let image_path = entries
        .choose(&mut thread_rng())
        .ok_or_else(|| Error::NoImage)?;
    let file_name = image_path
        .file_name()
        .ok_or_else(|| Error::NoFileName)?
        .to_str()
        .ok_or_else(|| Error::NoFileName)?;

    interaction
        .edit_response(
            &ctx,
            EditInteractionResponse::new()
                .embed(
                    CreateEmbed::new()
                        .title(format!("Good Night, {}!", interaction.user.name))
                        .attachment(file_name),
                )
                .attachments(EditAttachments::new().add(CreateAttachment::path(image_path).await?)),
        )
        .await?;

    Ok(())
}

pub async fn register(ctx: &Context) -> Result<()> {
    GuildId::new(COLLEGE_KINGS_GUILD_ID)
        .create_command(
            ctx,
            CreateCommand::new("good_night").description("Have a CK girl wish you good night"),
        )
        .await?;

    Ok(())
}