use std::error::Error as StdError;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serenity::async_trait;

use serenity::client::{Context, EventHandler};
use serenity::futures::StreamExt;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;

use serenity::model::interactions::{Interaction, InteractionResponseType};

use serenity::utils::MessageBuilder;

use crate::animal::Animal;
use crate::sound::Sound;

const QUIZ_STRING: &str = "quiz";

pub struct Handler;

impl Handler {
    async fn interaction_create(
        &self,
        context: Context,
        interaction: Interaction,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get the slash command, or return if it's not a slash command.
        let slash_command = if let Some(slash_command) = interaction.application_command() {
            slash_command
        } else {
            return Ok(());
        };

        if let Err(_e) = slash_command.channel_id.to_channel(&context).await {
            // warn!("Error getting channel: {:?}", e);
        };

        dbg!(&slash_command.data.name[..]);

        match &slash_command.data.name[..] {
            QUIZ_STRING => {
                // Get the current number of seconds since the epoch
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                // Load text from file question/q1.rs
                let mut file = File::open("questions/q1.rs")?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                slash_command
                    .create_interaction_response(&context.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message.content(
                                    MessageBuilder::new()
                                        .push("Does this code compile?")
                                        .push("```rust\n")
                                        .push(&contents)
                                        .push("```")
                                        .push(format!("Showing answer <t:{}:R>", now + 20))
                                        .build(),
                                )
                            })
                    })
                    .await?;
            }
            _ => {
                // warn!("should not happen");
                return Ok(());
            }
        }

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, context: Context, interaction: Interaction) {
        if let Err(_e) = self.interaction_create(context, interaction).await {
            // error!(?e, "Error while processing message");
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content != "animal" {
            return;
        }

        // Ask the user for its favorite animal
        let m = msg
            .channel_id
            .send_message(&ctx, |m| {
                m.content("Please select your favorite animal")
                    .components(|c| c.add_action_row(Animal::action_row()))
            })
            .await
            .unwrap();

        // Wait for the user to make a selection
        let mci = match m
            .await_component_interaction(&ctx)
            .timeout(Duration::from_secs(60 * 3))
            .await
        {
            Some(ci) => ci,
            None => {
                m.reply(&ctx, "Timed out").await.unwrap();
                return;
            }
        };

        // data.custom_id contains the id of the component (here "animal_select")
        // and should be used to identify if a message has multiple components.
        // data.values contains the selected values from the menu
        let animal = Animal::from_str(mci.data.values.get(0).unwrap()).unwrap();

        // Acknowledge the interaction and edit the message
        mci.create_interaction_response(&ctx, |r| {
            r.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|d| {
                    d.content(format!("You chose: **{}**\nNow choose a sound!", animal))
                        .components(|c| c.add_action_row(Sound::action_row()))
                })
        })
        .await
        .unwrap();

        // Wait for multiple interactions

        let mut cib = m
            .await_component_interactions(&ctx)
            .timeout(Duration::from_secs(60 * 3))
            .build();

        while let Some(mci) = cib.next().await {
            let sound = Sound::from_str(&mci.data.custom_id).unwrap();
            // Acknowledge the interaction and send a reply
            mci.create_interaction_response(&ctx, |r| {
                // This time we dont edit the message but reply to it
                r.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|d| {
                        // Make the message hidden for other users by setting `ephemeral(true)`.
                        d.ephemeral(true)
                            .content(format!("The **{}** says __{}__", animal, sound))
                    })
            })
            .await
            .unwrap();
        }

        // Delete the orig message or there will be dangling components
        m.delete(&ctx).await.unwrap()
    }

    async fn ready(&self, context: Context, ready: Ready) {
        let _name = ready.user.name;

        // Create the review command for the Veloren server
        if let Err(_e) = GuildId(345993194322001923)
            .create_application_command(&context.http, |command| {
                command.name("quiz").description("Start the quiz")
            })
            .await
        {
            // error!(?e, "Error while creating the review command");
        }
    }
}
