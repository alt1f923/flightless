mod daddies;
use daddies::mathdaddy;

use std::env;

use serenity::{
    client::Client,
    model::{
        channel::Message,
        user::OnlineStatus,
        gateway::{
            Activity,
            Ready
            },
        },
    prelude::{
        EventHandler, Context
    },
    framework::standard::{
        StandardFramework,
        CommandResult,
        macros::{
            command,
            group
        }
    }
};

group!({
    name: "general",
    options: {},
    commands: [solve],
});

struct Handler;
impl EventHandler for Handler {
    fn ready(&self, ctx: Context, _: Ready) {
        let status = OnlineStatus::Online;
        let activity = Activity::playing("rust good");
        ctx.set_presence(Some(activity), status);
        println!("connected");
    }
}

fn main() {
    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("FLIGHTLESS_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("f/")) // set the bot's prefix to "f/"
        .group(&GENERAL_GROUP));

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
fn solve(ctx: &mut Context, msg: &Message) -> CommandResult {
    let content = &msg.content;
    let (answer, statement, postfix_statement) = mathdaddy::solve(&content[8..content.len()].to_string());
    let mut reply = std::string::String::new();
    reply.push_str("your statement: ");
    reply = reply + &statement;
    reply.push_str("\npostfix statement: ");
    reply = reply + &postfix_statement;
    reply.push_str("\nsolution: ");
    reply = reply + &answer.to_string();

    println!("{:?}", msg.reply(ctx, &reply));


    Ok(())
}