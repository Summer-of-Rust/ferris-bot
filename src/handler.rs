use std::error::Error as StdError;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serenity::async_trait;

use serenity::client::{Context, EventHandler};
use serenity::futures::StreamExt;

use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;

use serenity::model::interactions::application_command::{
    ApplicationCommandInteractionDataOptionValue, ApplicationCommandOptionType,
};
use serenity::model::interactions::{Interaction, InteractionResponseType};

use serenity::prelude::Mentionable;
use serenity::utils::MessageBuilder;

use crate::question::QuestionTF;

const QUIZ_STRING: &str = "quiz";
const QUESTION_TIME: u64 = 30;

pub struct Handler;

impl Handler {
    async fn interaction_create(
        &self,
        ctx: Context,
        interaction: Interaction,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get the slash command, or return if it's not a slash command.
        let slash_command = if let Some(slash_command) = interaction.application_command() {
            slash_command
        } else {
            return Ok(());
        };

        let channel = match slash_command.channel_id.to_channel(&ctx).await {
            Ok(channel) => channel,
            Err(why) => {
                println!("Error getting channel: {:?}", why);
                return Ok(());
            }
        };

        // Match the different potential slash commands
        match &slash_command.data.name[..] {
            QUIZ_STRING => {
                // Get the number of the question
                let question_number = match slash_command
                    .data
                    .options
                    .get(0)
                    .expect("Expected int option")
                    .resolved
                    .as_ref()
                    .expect("Expected int object")
                {
                    ApplicationCommandInteractionDataOptionValue::Integer(i) => *i,
                    _ => {
                        println!("Expected int option");
                        return Ok(());
                    }
                };

                let answers = [
                    QuestionTF::True,
                    QuestionTF::True,
                    QuestionTF::False,
                    QuestionTF::False,
                    QuestionTF::True,
                    QuestionTF::True,
                    QuestionTF::False,
                ];

                if question_number > answers.len() as i64 {
                    println!("Question number out of bounds");
                    return Ok(());
                }

                // Get the current number of seconds since the epoch
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                // Load text from file question/q1.rs
                let mut file = File::open(format!("questions/q{}.rs", question_number))?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                // Ask the question
                slash_command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message
                                    .content(MessageBuilder::new().push("Starting Quiz!").build())
                            })
                    })
                    .await?;

                let m = channel
                    .id()
                    .send_message(&ctx, |m| {
                        m.content(
                            MessageBuilder::new()
                                .push("Does this code compile?")
                                .push("```rust\n")
                                .push(&contents)
                                .push("```")
                                .push(format!("Showing answer <t:{}:R>", now + QUESTION_TIME))
                                .build(),
                        )
                        .components(|c| c.add_action_row(QuestionTF::action_row()))
                    })
                    .await
                    .unwrap();

                // Wait for a responses within a certain amount of time
                let mut cib = m
                    .await_component_interactions(&ctx)
                    .timeout(Duration::from_secs(QUESTION_TIME))
                    .build();

                let mut correct_answers = Vec::new();

                while let Some(mci) = cib.next().await {
                    println!("{:?}", mci.data);
                    let question_choice = QuestionTF::from_str(&mci.data.custom_id).unwrap();

                    let member = mci.member.clone().unwrap();

                    if question_choice == answers[(question_number - 1) as usize] {
                        correct_answers.push(member);
                    }

                    // Acknowledge the interaction and send a reply
                    mci.create_interaction_response(&ctx, |r| {
                        // This time we dont edit the message but reply to it
                        r.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|d| {
                                // Make the message hidden for other users by setting `ephemeral(true)`.
                                d.ephemeral(true)
                                    .content(format!("You choose {}", question_choice))
                            })
                    })
                    .await
                    .unwrap();
                }

                m.delete(&ctx).await.unwrap();

                // Write a message with people who got the question right
                let _m = channel
                    .id()
                    .send_message(&ctx, |m| {
                        let mut builder = MessageBuilder::new();

                        builder.push("The following people got the question right:\n\n");

                        for member in correct_answers {
                            builder.push(member.mention()).push(" ");
                        }

                        m.content(
                            builder
                                .push("\n\nThe correct answer was:")
                                .push(answers[(question_number - 1) as usize].to_string())
                                .push("```rust\n")
                                .push(&contents)
                                .push("```")
                                .build(),
                        )
                    })
                    .await
                    .unwrap();
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

    async fn ready(&self, context: Context, ready: Ready) {
        let _name = ready.user.name;

        // Create the review command for the Veloren server
        if let Err(_e) = GuildId(968184259053518858)
            .create_application_command(&context.http, |command| {
                command
                    .name("quiz")
                    .description("Start the quiz")
                    .create_option(|option| {
                        option
                            .name("id")
                            .description("The question to ask")
                            .kind(ApplicationCommandOptionType::Integer)
                            .required(true)
                    })
            })
            .await
        {
            // error!(?e, "Error while creating the review command");
        }
    }
}
