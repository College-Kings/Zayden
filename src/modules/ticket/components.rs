use serenity::all::{ComponentInteraction, Context, CreateInputText, InputTextStyle};
use sqlx::{PgPool, Postgres};
use ticket::TicketComponent;

use crate::sqlx_lib::GuildTable;
use crate::Result;

use super::Ticket;

impl Ticket {
    pub async fn support_close(ctx: &Context, component: &ComponentInteraction) -> Result<()> {
        TicketComponent::support_close(ctx, component).await?;

        Ok(())
    }

    pub async fn support_faq(
        ctx: &Context,
        component: &ComponentInteraction,
        pool: &PgPool,
    ) -> Result<()> {
        TicketComponent::support_faq::<Postgres, GuildTable>(ctx, component, pool).await?;

        Ok(())
    }

    pub async fn ticket_create(ctx: &Context, component: &ComponentInteraction) -> Result<()> {
        let version =
            CreateInputText::new(InputTextStyle::Short, "Version", "version").placeholder("1.0.0");

        let additional = CreateInputText::new(
            InputTextStyle::Paragraph,
            "Additional Information",
            "additional",
        )
        .placeholder("Please provide any additional information that may help us assist you.")
        .required(false);

        TicketComponent::ticket_create(ctx, component, [version, additional]).await?;

        Ok(())
    }
}
