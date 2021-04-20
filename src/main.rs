use serenity::{client::{Client, Context, EventHandler}, prelude::TypeMapKey};
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
use std::collections::HashMap;
use std::time::Duration;

use pleco::{BitMove, Board, SQ, core::piece_move::{MoveFlag, PreMoveInfo}};

use ascii::{self, AsAsciiStr, AsAsciiStrError, Chars};

#[group]
#[commands(ping, startchess, move_piece)]
struct General;

/* 
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
*/

struct UserToBoard;

impl TypeMapKey for UserToBoard {
    type Value = HashMap<u64, Board>;
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(token)
        // .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    client.data.write().await.insert::<UserToBoard>(HashMap::new());

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

#[command("start")]
async fn startchess(ctx: &Context, msg: &Message) -> CommandResult {
    let mut lock = ctx.data.write().await;
    let map = lock.get_mut::<UserToBoard>().unwrap();

    if map.contains_key(msg.author.id.as_u64()) {
        let reaction_msg = msg.channel_id.say(ctx, "Previous game found, delete and add new game?").await?;
        reaction_msg.react(ctx, ReactionType::Unicode("✅".to_string())).await?;
        reaction_msg.react(ctx, ReactionType::Unicode("❌".to_string())).await?;

        if let Some(reaction) = reaction_msg.await_reaction(&ctx).timeout(Duration::from_secs(15)).author_id(msg.author.id).await {
            let _ = match reaction.as_inner_ref().emoji.as_data().as_str() {
                "✅" => { msg.channel_id.say(ctx, "Starting a new game...").await?; map.remove(msg.author.id.as_u64()); } ,
                _ => { msg.channel_id.say(ctx, "Aborted".to_string()).await?; return Ok(()); } ,
            };
        } else {
            msg.reply(ctx, "Aborted".to_string()).await?;
            return Ok(());
        }
    }
    
    let new_board = Board::start_pos();
    
    msg.channel_id.say(ctx, format!("```\n{}\n```", new_board.pretty_string())).await?;

    map.insert(msg.author.id.as_u64().clone(), new_board);

    Ok(())
}

#[command("move")]
async fn move_piece(ctx: &Context, msg: &Message) -> CommandResult {
    let mut lock = ctx.data.write().await;
    let map = lock.get_mut::<UserToBoard>().unwrap();

    if let Some(board) = map.get_mut(msg.author.id.as_u64()) {
        let spaces: Vec<Result<Chars, AsAsciiStrError>> = msg.content.split(' ').skip(1).map(|x| x.as_ascii_str().map(|x| x.chars())).collect();
        
        if spaces.iter().any(|x| x.is_err()) {
            return Ok(());
        }

        let spaces_u64 = spaces.iter()
            .map(|x| x.unwrap())
            .map(|x| {
                let result = 0u8;
                for y in x {
                    if y >= 'A' && y <= 'H' {
                        result += y.as_byte();
                    }
                }
            }
        );

        let valids = board.generate_moves();
        
        let proposed = BitMove::init(PreMoveInfo {
            src: SQ::from("E2"),
            dst: SQ::from("E4"),
            flags: MoveFlag::DoublePawnPush,
        });

        if valids.contains(&proposed) {
            board.apply_move(proposed);

            msg.channel_id.say(ctx, format!("```\n{}\n```", board.pretty_string())).await?;
        }
        else {
            msg.channel_id.say(ctx, "Invalid move".to_string()).await?;
        }
    }
    else {
        msg.channel_id.say(ctx, "You don't currently have a game running".to_string()).await?;
    }

    Ok(())
}
