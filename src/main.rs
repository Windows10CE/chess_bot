use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::{
    async_trait,
    model::channel::ReactionType,
};

use std::env;

use pleco::Board;

#[group]
#[commands(ping, startchess)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, new_message: Message) {
        if new_message.content.starts_with('~') {
            new_message
                .react(ctx, ReactionType::Unicode("😔".to_string()))
                .await
                .unwrap();
        }
    }
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn startchess(ctx: &Context, msg: &Message) -> CommandResult {
    let board = Board::start_pos();

    msg.reply(ctx, format!("```\n{}\n```", board.pretty_string())).await?;

    Ok(())
}
