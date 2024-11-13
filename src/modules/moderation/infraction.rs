use std::cmp;

use async_trait::async_trait;
use chrono::{Months, TimeDelta, Utc};
use serenity::all::{
    CommandInteraction, CommandOptionType, CreateEmbed, Message, Ready, ResolvedValue, User, UserId,
};
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::prelude::GuildId;
use serenity::model::{Permissions, Timestamp};
use serenity::prelude::Context;
use sqlx::{PgPool, Pool, Postgres};
use zayden_core::{parse_options, SlashCommand};

use crate::sqlx_lib::PostgresPool;
use crate::utils::{dm_user_embed, embed_response};
use crate::{Error, Result};

use super::{InfractionKind, InfractionRow};

pub struct Infraction;

#[async_trait]
impl SlashCommand<Error> for Infraction {
    async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
        let guild_id = interaction.guild_id.ok_or_else(|| Error::NotInGuild)?;

        let options = interaction.data.options();
        let options = parse_options(&options);

        let user = match options.get("user") {
            Some(ResolvedValue::User(user, _)) => *user,
            _ => unreachable!("User option is required"),
        };

        let points = match options.get("points") {
            Some(ResolvedValue::Integer(points)) => *points as i32,
            _ => 1,
        };

        let reason = match options.get("reason") {
            Some(ResolvedValue::String(reason)) => *reason,
            _ => "No reason provided.",
        };

        let pool = PostgresPool::get(ctx).await;

        let infractions = InfractionRow::user_infractions(&pool, user.id, false).await?;

        let six_months_age = Utc::now()
            .checked_sub_months(Months::new(6))
            .ok_or_else(|| Error::TimeDelta)?
            .naive_utc();

        let infractions: Vec<_> = infractions
            .iter()
            .filter(|infraction| infraction.created_at >= six_months_age)
            .collect();
        let infraction_count: i32 = infractions
            .into_iter()
            .map(|infraction| infraction.points)
            .sum();
        let infraction_count = cmp::min(infraction_count + points, 5);

        let embed = match infraction_count {
            1 => {
                warn(
                    ctx,
                    &pool,
                    guild_id,
                    user,
                    &interaction.user,
                    points,
                    reason,
                )
                .await?
            }
            2 => {
                mute(
                    ctx,
                    &pool,
                    guild_id,
                    user,
                    &interaction.user,
                    TimeDelta::try_hours(1).ok_or_else(|| Error::TimeDelta)?,
                    points,
                    reason,
                )
                .await?
            }
            3 => {
                mute(
                    ctx,
                    &pool,
                    guild_id,
                    user,
                    &interaction.user,
                    TimeDelta::try_hours(8).ok_or_else(|| Error::TimeDelta)?,
                    points,
                    reason,
                )
                .await?
            }
            4 => {
                mute(
                    ctx,
                    &pool,
                    guild_id,
                    user,
                    &interaction.user,
                    TimeDelta::try_days(28).ok_or_else(|| Error::TimeDelta)?,
                    points,
                    reason,
                )
                .await?
            }
            5 => {
                ban(
                    ctx,
                    &pool,
                    guild_id,
                    user,
                    &interaction.user,
                    points,
                    reason,
                )
                .await?
            }
            _ => unreachable!("Invalid infraction count"),
        };

        embed_response(ctx, interaction, embed).await?;

        Ok(())
    }

    fn register(_ctx: &Context, _ready: &Ready) -> Result<CreateCommand> {
        let command = CreateCommand::new("infraction")
            .name("infraction")
            .description("Warn, mute, or ban a user")
            .default_member_permissions(Permissions::MODERATE_MEMBERS)
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::User,
                    "user",
                    "The user to warn, mute, or ban",
                )
                .required(true),
            )
            .add_option(CreateCommandOption::new(
                CommandOptionType::Integer,
                "points",
                "The number of infractions to give the user",
            ))
            .add_option(CreateCommandOption::new(
                CommandOptionType::String,
                "reason",
                "The reason for the infraction",
            ));

        Ok(command)
    }
}

async fn send_user_message(
    ctx: &Context,
    user_id: UserId,
    kind: InfractionKind,
    desc: impl Into<String>,
) -> Result<Message> {
    let title = match kind {
        InfractionKind::Warn => "You have been warned",
        InfractionKind::Mute => "You have been muted",
        InfractionKind::Kick => "You have been kicked",
        InfractionKind::SoftBan => "You have been softbanned",
        InfractionKind::Ban => "You have been banned",
    };

    let embed = CreateEmbed::new().title(title).description(desc);
    let message = dm_user_embed(ctx, user_id, embed).await?;

    Ok(message)
}

async fn warn<'a>(
    ctx: &Context,
    pool: &PgPool,
    guild_id: GuildId,
    user: &User,
    moderator: &User,
    points: i32,
    reason: &str,
) -> Result<CreateEmbed> {
    InfractionRow::new(
        user.id,
        &user.name,
        guild_id,
        InfractionKind::Warn,
        moderator,
        points,
        reason,
    )?
    .create(pool)
    .await?;

    let desc = if reason == "No reason provided." {
        format!(
            "You have been warned in {}.",
            guild_id.to_partial_guild(ctx).await?.name
        )
    } else {
        format!(
            "You have been warned in {} for the following reason:\n{}",
            guild_id.to_partial_guild(ctx).await?.name,
            reason
        )
    };

    send_user_message(ctx, user.id, InfractionKind::Warn, desc).await?;

    let mut embed = CreateEmbed::new().title(format!("{} has been warned", user.name));
    if reason != "No reason provided." {
        embed = embed.description(reason);
    }
    Ok(embed)
}

#[allow(clippy::too_many_arguments)]
async fn mute<'a>(
    ctx: &Context,
    pool: &Pool<Postgres>,
    guild_id: GuildId,
    user: &User,
    moderator: &User,
    duration: TimeDelta,
    points: i32,
    reason: &str,
) -> Result<CreateEmbed> {
    let mut member = guild_id.member(ctx, user.id).await?;

    let timestamp = (Utc::now() + duration).timestamp();
    member
        .disable_communication_until_datetime(ctx, Timestamp::from_unix_timestamp(timestamp)?)
        .await?;

    InfractionRow::new(
        user.id.get(),
        &user.name,
        guild_id.get(),
        InfractionKind::Ban,
        moderator,
        points,
        reason,
    )?
    .create(pool)
    .await?;

    let days = duration.num_days();
    let hours = duration.num_hours() % 24;

    let mut duration_str = String::new();
    if days > 0 {
        duration_str.push_str(&format!("{} day", days));
        if days > 1 {
            duration_str.push('s');
        }
    } else if hours > 0 {
        duration_str.push_str(&format!("{} hour", hours));
        if hours > 1 {
            duration_str.push('s');
        }
    }

    let desc: String = if reason == "No reason provided." {
        format!(
            "You have been muted in {} for {}.",
            guild_id.to_partial_guild(ctx).await?.name,
            duration_str
        )
    } else {
        format!(
            "You have been muted in {} for {}\n{}",
            guild_id.to_partial_guild(ctx).await?.name,
            duration_str,
            reason
        )
    };

    send_user_message(ctx, user.id, InfractionKind::Mute, &desc).await?;

    let mut embed = CreateEmbed::new().title(format!("{} has been muted", user.name));
    if reason != "No reason provided." {
        embed = embed.description(reason);
    }
    Ok(embed)
}

async fn ban<'a>(
    ctx: &Context,
    pool: &Pool<Postgres>,
    guild_id: GuildId,
    user: &User,
    moderator: &User,
    points: i32,
    reason: &str,
) -> Result<CreateEmbed> {
    let member = guild_id.member(ctx, user.id).await?;

    let desc = if reason == "No reason provided." {
        format!(
            "You have been banned from {}.",
            guild_id.to_partial_guild(ctx).await?.name
        )
    } else {
        format!(
            "You have been banned from {} for the following reason:\n{}",
            guild_id.to_partial_guild(ctx).await?.name,
            reason
        )
    };

    send_user_message(ctx, user.id, InfractionKind::Ban, desc).await?;

    member.ban_with_reason(ctx, 1, reason).await?;

    InfractionRow::new(
        user.id,
        &user.name,
        guild_id,
        InfractionKind::Ban,
        moderator,
        points,
        reason,
    )?
    .create(pool)
    .await?;

    let mut embed = CreateEmbed::new().title(format!("{} has been banned", user.name));
    if reason != "No reason provided." {
        embed = embed.description(reason);
    }
    Ok(embed)
}