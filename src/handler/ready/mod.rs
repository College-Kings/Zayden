mod message_updates;

use serenity::all::{Context, OnlineStatus, Ready};

use message_updates::update_messages;

use crate::cron::start_cron_jobs;
use crate::{global_commands, guild_commands, Result};

pub async fn ready(ctx: &Context, ready: Ready) -> Result<()> {
    println!("{} is connected!", ready.user.name);

    ctx.set_presence(None, OnlineStatus::Online);

    // TODO: Load Commands

    tokio::try_join!(
        guild_commands::register(ctx),
        global_commands::register(ctx),
        update_messages(ctx)
    )?;

    let ctx_clone = ctx.clone();
    tokio::spawn(async move { start_cron_jobs(ctx_clone).await });

    Ok(())
}
