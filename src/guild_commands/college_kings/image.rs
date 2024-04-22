use std::path::PathBuf;

use crate::image_cache::ImageCache;
use crate::{Error, Result};
use rand::seq::{IteratorRandom, SliceRandom};
use rand::thread_rng;
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateCommand, CreateEmbed, EditAttachments,
    EditInteractionResponse,
};

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
    interaction.defer(&ctx).await?;

    let guild_roles = interaction
        .guild_id
        .ok_or_else(|| Error::NoGuild)?
        .roles(&ctx)
        .await?;

    let member_roles = &interaction
        .member
        .as_ref()
        .ok_or_else(|| Error::NoMember)?
        .roles;

    let image_map = {
        let data = ctx.data.read().await;
        let image_cache = data
            .get::<ImageCache>()
            .ok_or_else(|| Error::DataNotFound)?;
        image_cache.character_map.clone()
    };

    let entries: Vec<&PathBuf> = member_roles
        .iter()
        .map(|role_id| -> Result<&str> {
            let role = guild_roles.get(role_id).ok_or_else(|| Error::NoRole)?;
            Ok(&role.name)
        })
        .filter_map(|name| {
            let name = name.ok()?.split('\'').next()?;
            image_map.get(&name.to_lowercase())
        })
        .flatten()
        .collect();

    let image_path = match entries.choose(&mut thread_rng()) {
        Some(path) => *path,
        None => image_map
            .values()
            .flat_map(|v| v.iter())
            .choose(&mut thread_rng())
            .ok_or_else(|| Error::NoImage)?,
    };
    let file_name = image_path
        .file_name()
        .ok_or_else(|| Error::NoFileName)?
        .to_str()
        .ok_or_else(|| Error::NoFileName)?;

    interaction
        .edit_response(
            &ctx,
            EditInteractionResponse::new()
                .embed(CreateEmbed::new().attachment(file_name))
                .attachments(EditAttachments::new().add(CreateAttachment::path(image_path).await?)),
        )
        .await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("image").description("Get a random image from the image cache")
}