use dotenv::dotenv;
use poise::serenity_prelude as serenity;

mod commands;
use crate::commands::quiz;

mod question;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
pub struct Data {}

#[derive(Debug, poise::Modal)]
#[allow(dead_code)] // fields only used for Debug print
struct MyModal {
    first_input: String,
    #[paragraph]
    second_input: Option<String>,
}
#[poise::command(slash_command)]
async fn modal(ctx: poise::ApplicationContext<'_, crate::Data, crate::Error>) -> Result<(), Error> {
    use poise::Modal as _;

    let data = MyModal::execute(ctx).await?;
    println!("Got data: {:?}", data);

    Ok(())
}

/// Registers or unregisters application commands in this guild or globally
#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let framework = poise::Framework::build()
        .options(poise::FrameworkOptions {
            commands: vec![register(), modal(), quiz::quiz()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            listener: |_ctx, event, _framework, _data| {
                Box::pin(async move {
                    println!("Got an event in listener: {:?}", event.name());
                    Ok(())
                })
            },
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }));

    framework.run().await.unwrap();
}
