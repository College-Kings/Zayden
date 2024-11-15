use std::time::Duration;

use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
use serenity::all::{
    ButtonStyle, CommandInteraction, CommandOptionType, Context, CreateButton, CreateCommand,
    CreateCommandOption, CreateEmbed, CreateInteractionResponse, EditInteractionResponse, GuildId,
    Member, PermissionOverwrite, PermissionOverwriteType, Permissions, Ready, ResolvedValue,
};
use sqlx::PgPool;
use zayden_core::SlashCommand;

use crate::guilds::{ServersTable, ServersTableError};
use crate::handler::OnReady;
use crate::sqlx_lib::PostgresPool;
use crate::utils::message_response;
use crate::{Error, Result};

pub struct Sleep;

#[async_trait]
impl SlashCommand<Error> for Sleep {
    async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
        interaction.defer_ephemeral(ctx).await?;

        let hours: u64 = match &interaction.data.options()[0].value {
            ResolvedValue::Integer(hours) => match (*hours).try_into() {
                Ok(hours) => hours,
                Err(_) => {
                    message_response(ctx, interaction, "Hours must be a positive integer.").await?;
                    return Ok(());
                }
            },
            _ => unreachable!("Hours option is required and must be an integer."),
        };

        let embed = CreateEmbed::new().description("This command will remove your ability to view any channels in the server until the time limit has expired or the zayden-bot restarts. This cannot be undone at this time.\nAre you sure you want to continue?");

        let confirm_button = CreateButton::new("sleep_confirm")
            .label("Confirm")
            .style(ButtonStyle::Success);
        let cancel_button = CreateButton::new("sleep_cancel")
            .label("Cancel")
            .style(ButtonStyle::Danger);

        let message = interaction
            .edit_response(
                ctx,
                EditInteractionResponse::new()
                    .embed(embed)
                    .button(confirm_button)
                    .button(cancel_button),
            )
            .await?;

        if let Some(interaction) = message
            .await_component_interaction(ctx)
            .timeout(Duration::from_secs(60))
            .await
        {
            interaction
                .create_response(ctx, CreateInteractionResponse::Acknowledge)
                .await?;

            if interaction.data.custom_id == "sleep_confirm" {
                let member = interaction.member.unwrap();

                let ctx_clone = ctx.clone();
                tokio::spawn(async move { sleep_user(ctx_clone, member, hours).await });
            }
        };

        Ok(())
    }

    fn register(_ctx: &Context, _ready: &Ready) -> Result<CreateCommand> {
        let command = CreateCommand::new("sleep")
            .description("Remove the distraction of discord for a while")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "hours",
                    "The number of hours to sleep for",
                )
                .required(true),
            );

        Ok(command)
    }
}

#[async_trait]
impl OnReady for Sleep {
    async fn on_ready(ctx: Context, ready: Ready) -> Result<()> {
        let pool = PostgresPool::get(&ctx).await;

        for guild in ready.guilds {
            foo(&ctx, &pool, guild.id).await?;
        }

        Ok(())
    }
}

async fn foo(ctx: &Context, pool: &PgPool, guild_id: GuildId) -> Result<()> {
    let sleep_role_id = ServersTable::get_row(pool, guild_id)
        .await?
        .ok_or(ServersTableError::ServerNotFound)?
        .get_sleep_role_id()?;

    let channels = guild_id.channels(&ctx).await?;
    for channel in channels.values() {
        if channel.name == "sleeping" {
            continue;
        }

        channel
            .create_permission(
                &ctx,
                PermissionOverwrite {
                    allow: Permissions::empty(),
                    deny: Permissions::VIEW_CHANNEL,
                    kind: PermissionOverwriteType::Role(sleep_role_id),
                },
            )
            .await?;
    }

    let mut members = guild_id.members_iter(&ctx).boxed();

    while let Some(member) = members.try_next().await? {
        if member.roles.contains(&sleep_role_id) {
            member.remove_role(&ctx, sleep_role_id).await?;
        }
    }

    Ok(())
}

async fn sleep_user(ctx: Context, member: Member, hours: u64) -> Result<()> {
    let pool = PostgresPool::get(&ctx).await;

    let sleep_role_id = ServersTable::get_row(&pool, member.guild_id)
        .await?
        .ok_or(ServersTableError::ServerNotFound)?
        .get_sleep_role_id()?;

    drop(pool);

    member.add_role(&ctx, sleep_role_id).await?;

    tokio::time::sleep(Duration::from_secs(hours * 60 * 60)).await;

    println!("Waking up {}", member.user.name);

    member.remove_role(&ctx, sleep_role_id).await?;

    Ok(())
}
