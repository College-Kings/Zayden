use crate::{utils::message_response, COLLEGE_KINGS_GUILD_ID};
use serenity::all::{
    ChannelId, CommandInteraction, Context, CreateCommand, CreateEmbed, CreateMessage,
    GuildChannel, GuildId, ReactionType,
};
use std::time;

use crate::{Error, Result};

const SUGGESTION_CHANNEL_ID: u64 = 1068790374996377671;

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
    let start_time = time::Instant::now();

    let guild_id = interaction.guild_id.ok_or_else(|| Error::NoGuild)?;

    let suggestion_channel = ChannelId::new(SUGGESTION_CHANNEL_ID);

    let active_guild_threads = guild_id.get_active_threads(&ctx).await?;
    let threads: Vec<GuildChannel> = active_guild_threads
        .threads
        .into_iter()
        .chain(
            suggestion_channel
                .get_archived_public_threads(&ctx, None, None)
                .await?
                .threads,
        )
        .filter(|thread| {
            thread
                .parent_id
                .is_some_and(|id| id == SUGGESTION_CHANNEL_ID)
        })
        .collect();

    let mut thread_reaction_counts = Vec::with_capacity(threads.len());

    for thread in threads {
        let reactions = thread
            .reaction_users(
                &ctx,
                thread.id.get(),
                ReactionType::Unicode("👍".to_string()),
                Some(100),
                None,
            )
            .await?;
        thread_reaction_counts.push((thread, reactions.len()));
    }

    thread_reaction_counts.sort_by(|a, b| b.1.cmp(&a.1));

    let elapsed_time = start_time.elapsed();

    let mut embed = CreateEmbed::new()
        .title("Top 10 suggestions")
        .description("Here are the top 10 suggestions, sorted by votes.");

    for (i, (thread, count)) in thread_reaction_counts[..10].iter().enumerate() {
        embed = embed.field(
            format!("{}. 👍: {}", i + 1, count),
            format!("Link: {}", thread),
            false,
        );
    }

    interaction
        .user
        .dm(&ctx, CreateMessage::new().add_embed(embed))
        .await?;

    message_response(
        ctx,
        interaction,
        &format!(
            "Suggestions fetched. Took {} seconds",
            elapsed_time.as_secs()
        ),
    )
    .await?;

    Ok(())
}

pub async fn register(ctx: &Context) -> Result<()> {
    GuildId::new(COLLEGE_KINGS_GUILD_ID)
        .create_command(
            ctx,
            CreateCommand::new("fetch_suggestions")
                .description("Fetch suggestions from the suggestion channel"),
        )
        .await?;

    Ok(())
}