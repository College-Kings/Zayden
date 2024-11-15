use reqwest::Client;
use serenity::all::{
    ButtonStyle, Context, CreateButton, CreateChannel, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage, InputText, Mentionable, ModalInteraction,
    PermissionOverwrite, PermissionOverwriteType, Permissions,
};

use crate::modules::patreon::PatreonUser;
use crate::{
    guilds::{college_kings::RENDER_REQUESTS_CHANNEL_ID, college_kings_team::MESSY_USER_ID},
    Error, Result,
};

use super::parse_modal_data;

pub async fn run(ctx: &Context, modal: &ModalInteraction) -> Result<()> {
    let guild_id = modal.guild_id.ok_or_else(|| Error::NotInGuild)?;

    let client = Client::new();

    let data = parse_modal_data(&modal.data.components);
    let user = match data.get("email") {
        Some(InputText {
            value: Some(email), ..
        }) => PatreonUser::get(&client, email, false).await?,
        _ => PatreonUser::get(&client, modal.user.id, false).await?,
    };

    let character = match data.get("character") {
        Some(InputText {
            value: Some(value), ..
        }) => value.as_str(),
        _ => unreachable!("Character input is required"),
    };

    let prop = match data.get("prop") {
        Some(InputText {
            value: Some(value), ..
        }) => value.as_str(),
        _ => "No prop specified.",
    };

    let location = match data.get("location") {
        Some(InputText {
            value: Some(value), ..
        }) => value,
        _ => "No location specified.",
    };

    let description = match data.get("description") {
        Some(InputText {
            value: Some(value), ..
        }) => value,
        _ => "No description specified.",
    };

    if user.tier < 50 {
        modal
            .create_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("You must be at least a $50 patron to use this feature.\nThe cache is updated every 24 hours. If you've recently upgraded, please wait a day and try again or contact us through the support channel.").ephemeral(true),
                ),
            )
            .await?;

        return Ok(());
    }

    let request = CreateEmbed::new()
        .title("Render Request")
        .description(description)
        .fields(vec![
            ("Tier", format!("${}", user.tier).as_str(), true),
            ("Character", character, false),
            ("Prop", prop, false),
            ("Location", location, false),
            ("Description", description, false),
        ]);

    let channel_name: String = format!("{}丨{}", chrono::Utc::now().format("%b"), &modal.user.name)
        .chars()
        .take(100)
        .collect();

    let category_id = RENDER_REQUESTS_CHANNEL_ID
        .to_channel(ctx)
        .await?
        .guild()
        .unwrap()
        .parent_id
        .unwrap();

    let permissions = vec![
        PermissionOverwrite {
            allow: Permissions::empty(),
            deny: Permissions::VIEW_CHANNEL,
            kind: PermissionOverwriteType::Role(guild_id.everyone_role()),
        },
        PermissionOverwrite {
            allow: Permissions::VIEW_CHANNEL | Permissions::MANAGE_CHANNELS,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Member(MESSY_USER_ID),
        },
        PermissionOverwrite {
            allow: Permissions::VIEW_CHANNEL,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Member(modal.user.id),
        },
    ];

    let channel = guild_id
        .create_channel(
            ctx,
            CreateChannel::new(channel_name)
                .category(category_id)
                .nsfw(true)
                .permissions(permissions),
        )
        .await?;

    channel
        .send_message(
            ctx,
            CreateMessage::new()
                .content(format!(
                    "{} {}",
                    MESSY_USER_ID.mention(),
                    modal.user.mention()
                ))
                .embed(request)
                .button(
                    CreateButton::new("delete_channel")
                        .label("Delete Channel")
                        .style(ButtonStyle::Danger),
                ),
        )
        .await?;

    modal
        .create_response(ctx, CreateInteractionResponse::Acknowledge)
        .await?;

    Ok(())
}
