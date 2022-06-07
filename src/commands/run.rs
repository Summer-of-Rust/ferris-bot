use crate::model::runnable::*;
use crate::Error;


use serenity::utils::MessageBuilder;

/// Given some stdout or stderr data, format it so that it can be rendered by discord
fn format_output(response: String) -> String {
    if response.len() < 1990 {
        // Response falls within size constraints
        return format!("```\n{}\n```", response);
    } else {
        // we trim to 1981 chars because [TRIMMED] is 9 chars
        let short_repsonse = &response[0..1981]; // TODO: maybe do this in place with a mutable string
        return format!("```{}[TRIMMED]```", short_repsonse);
    }
}

async fn reply(
    ctx: poise::ApplicationContext<'_, crate::Data, crate::Error>,
    code: String,
    reply_text: String,
) -> Result<(), Error> {
    let interaction = ctx.interaction.unwrap();
    let channel = match interaction.channel_id.to_channel(&ctx.discord.http).await {
        Ok(channel) => channel,
        Err(why) => {
            println!("Error getting channel: {:?}", why);
            return Ok(());
        }
    };

    channel
        .id()
        .send_message(&ctx.discord.http, |m| {
            m.content(
                MessageBuilder::new()
                    .mention(&interaction.member.clone().unwrap())
                    .push("Ran")
                    .push(code)
                    .push("Output")
                    .push(reply_text)
                    .build(),
            )
        })
        .await?;
    Ok(())
}

#[derive(Debug, poise::Modal)]
#[allow(dead_code)] // fields only used for Debug print
struct RunModal {
    #[name = "Code you want to run"]
    #[placeholder = "fn main() {\n    println!(\"Hello, world!\");\n}"]
    #[paragraph]
    code_to_run: String,
}

/// Runs whatever code you throw at it
#[poise::command(slash_command)]
pub async fn run(
    ctx: poise::ApplicationContext<'_, crate::Data, crate::Error>,
) -> Result<(), Error> {
    use poise::Modal as _;

    let _channel = match ctx
        .interaction
        .unwrap()
        .channel_id
        .to_channel(&ctx.discord.http)
        .await
    {
        Ok(channel) => channel,
        Err(why) => {
            println!("Error getting channel: {:?}", why);
            return Ok(());
        }
    };

    let modal_data = RunModal::execute(ctx).await?;
    let raw_code = modal_data.code_to_run;

    // This leverages the runnable trait we created for executing arbitrary strings of code
    let run_result = raw_code.run().await;

    match run_result {
        Ok(output) => {
            let mut stdout = String::new();
            let mut stderr = String::new();

            if !output.stdout.is_empty() {
                stdout = match String::from_utf8(output.stdout) {
                    Ok(v) => v,
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                };

                println!("Got stdout\n\"{}\"", stdout);
            } else {
                println!("No stdout");
            }

            if !output.stderr.is_empty() {
                stderr = match String::from_utf8(output.stderr) {
                    Ok(v) => v,
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                };
                println!("Got stderr \"{}\"", stderr);
            } else {
                println!("No stderr");
            }

            // TODO: better response classification
            // in the original rustbot we used reactions to indicate successful or failed compilation
            // or timeouts. With discord's command framework, it's a little more tricky.
            // For now we just use a canned response for everything, in the future it would be nice to add more
            // detailed responses for each type of response.

            // Check to see if the response was nothing
            if !stdout.is_empty() && stderr.is_empty() {
                println!("Response: has stdout, no stderr");
                //msg.react(&ctx, CHECK_MARK_EMOJI).await?;
                //msg.reply(&ctx, format_output(stdout)).await?;
                reply(ctx, format_output(raw_code), format_output(stdout)).await?;
            } else if stdout.is_empty() && !stderr.is_empty() {
                println!("Response: no stdout, has stderr");
                // Had stderr, no stdout
                //msg.react(&ctx, CROSS_MARK_EMOJI).await?;
                //msg.reply(&ctx, format_output(stderr)).await?;
                reply(ctx, format_output(raw_code), format_output(stderr)).await?;
            } else {
                println!("Response: no stdout, no stderr");
                //msg.react(&ctx, CHECK_MARK_EMOJI).await?;
                reply(
                    ctx,
                    format_output(raw_code),
                    "Your program had no output to STDOUT or STDERR".to_owned(),
                )
                .await?;
            }
        }
        Err(error) => {
            // TODO: find out ways this can blow up
            //println!("TIMEOUT on {}'s code", interaction.);
            match error.kind() {
                _timed_out => {
                    // Took too long to run, complain to user
                    //msg.react(&ctx, CROSS_MARK_EMOJI).await?;
                    //msg.react(&ctx, CLOCK_EMOJI).await?;
                    reply(
                        ctx,
                        format_output(raw_code),
                        "Your program took too long to run.".to_owned(),
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}
