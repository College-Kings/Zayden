use futures::{StreamExt, TryStreamExt};
use lazy_static::lazy_static;
use serenity::all::{
    ActionRowComponent, AutoArchiveDuration, ButtonKind, ChannelType, Context, CreateButton,
    CreateEmbed, CreateEmbedAuthor, CreateInteractionResponse, CreateMessage, CreateThread,
    InputText, Mentionable, ModalInteraction, RoleId,
};
use std::collections::HashMap;

use crate::{Error, Result};

use super::parse_modal_data;

lazy_static! {
    static ref ROLE_MAP: HashMap<&'static str, RoleId> = {
        let mut map = HashMap::new();
        map.insert("narrative", RoleId::new(963441815468539966));
        map.insert("programming", RoleId::new(963441811555254333));
        map.insert("transcribing", RoleId::new(1051653455460184114));
        map.insert("art", RoleId::new(963439862185345114));
        map
    };
}

pub async fn run(ctx: &Context, modal: &ModalInteraction) -> Result<()> {
    let data = parse_modal_data(&modal.data.components);
    let app_name = match data.get("app_name") {
        Some(InputText {
            value: Some(value), ..
        }) => value,
        _ => unreachable!("App Name input is required"),
    };

    let episode = match data.get("episode") {
        Some(InputText {
            value: Some(value), ..
        }) => value,
        _ => unreachable!("Version input is required"),
    };

    let scene = match data.get("scene") {
        Some(InputText {
            value: Some(value), ..
        }) => value,
        _ => unreachable!("Scene input is required"),
    };

    let request = match data.get("request") {
        Some(InputText {
            value: Some(value), ..
        }) => value,
        _ => unreachable!("Request input is required"),
    };

    let affected_teams = match data.get("teams") {
        Some(InputText {
            value: Some(value), ..
        }) => value,
        _ => "",
    };

    let teams: Vec<String> = affected_teams
        .split(", ")
        .map(|t| {
            ROLE_MAP
                .get(t.to_lowercase().trim())
                .map(|role| role.mention().to_string())
                .unwrap_or(t.trim().to_string())
        })
        .collect();

    let message = modal
        .channel_id
        .send_message(
            ctx,
            CreateMessage::default().content(format!(
                "{} - {} - {}\n{}",
                app_name,
                episode,
                scene,
                teams.join(" "),
            )),
        )
        .await?;

    let embed = CreateEmbed::default()
        .title(format!("{} - {} - {}", app_name, episode, scene))
        .description(request)
        .field("Affected Teams", teams.join(" "), true)
        .author(CreateEmbedAuthor::new(&modal.user.name));

    let channels = modal
        .guild_id
        .ok_or_else(|| Error::NotInGuild)?
        .channels(ctx)
        .await?;
    let channel = channels
        .get(&modal.channel_id)
        .ok_or_else(|| Error::NoChannel)?;

    let thread = channel
        .create_thread_from_message(
            ctx,
            message,
            CreateThread::new(format!("{} - {} - {}", app_name, episode, scene))
                .auto_archive_duration(AutoArchiveDuration::OneWeek)
                .invitable(true)
                .kind(ChannelType::PublicThread),
        )
        .await?;

    thread
        .send_message(ctx, CreateMessage::default().embed(embed))
        .await?;

    modal
        .create_response(ctx, CreateInteractionResponse::Acknowledge)
        .await?;

    resend_button_message(ctx, modal).await?;

    Ok(())
}

async fn resend_button_message(ctx: &Context, interaction: &ModalInteraction) -> Result<()> {
    let mut messages = interaction.channel_id.messages_iter(ctx).boxed();

    while let Some(message) = messages.try_next().await? {
        if let Some(ActionRowComponent::Button(b)) = message
            .components
            .first()
            .and_then(|c| c.components.first())
        {
            if let ButtonKind::NonLink { custom_id, .. } = &b.data {
                if custom_id == "production_request" {
                    message.delete(ctx).await?;
                    break;
                }
            }
        }
    }

    interaction
        .channel_id
        .send_message(
            ctx,
            CreateMessage::default()
                .button(CreateButton::new("production_request").label("Production Request")),
        )
        .await?;

    Ok(())
}
