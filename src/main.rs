use serenity::{client::{Client, Context}, prelude::TypeMapKey};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::model::channel::ReactionType;

use std::env;
use std::collections::HashMap;
use std::time::Duration;

use pleco::{BitMove, Board, PieceType, SQ, core::piece_move::{MoveFlag, PreMoveInfo}};

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

    map.insert(msg.channel_id.as_u64().clone(), new_board);

    Ok(())
}

#[command("move")]
async fn move_piece(ctx: &Context, msg: &Message) -> CommandResult {
    let mut lock = ctx.data.write().await;
    let map = lock.get_mut::<UserToBoard>().unwrap();

    if let Some(board) = map.get_mut(msg.channel_id.as_u64()) {
        let spaces: Vec<Vec<char>> = msg.content.split(' ').skip(1).map(|x| x.to_uppercase().chars().filter(|y| (*y >= 'A' && *y <= 'H') || (*y >= '1' && *y <= '8')).collect()).collect();

        if spaces.len() != 2 || spaces.iter().any(|x| x.len() != 2) {
            msg.channel_id.say(ctx, "Invalid move".to_string()).await?;
            return Ok(());
        }

        let spaces_u8: Vec<u8> = spaces.iter().map(|space| {
            space.iter().map(|x| {
                let mut utf8: [u8; 4] = [0; 4];
                x.encode_utf8(&mut utf8);
                let result = utf8[0];
                if result >= 65u8 && result <= 72u8 {
                    return result - 65u8;
                }
                else {
                    return (result - 49u8) * 8;
                }
            })
            .sum()
        })
        .collect();

        let source = SQ::from(spaces_u8[0]);
        let dest = SQ::from(spaces_u8[1]);
        let source_piece = board.piece_at_sq(source);
        let owner = source_piece.player();
        
        match owner {
            Some(player) => if player != board.turn() { msg.channel_id.say(ctx, "Invalid move".to_string()).await?; return Ok(()); }
            None => { msg.channel_id.say(ctx, "Invalid move".to_string()).await?; return Ok(()); }
        }

        let dest_piece = board.piece_at_sq(dest);
        let distance = source.distance(dest);

        let mut move_type = MoveFlag::QuietMove;

        if source_piece.type_of() == PieceType::K && distance > 1 {
            move_type = MoveFlag::Castle { king_side: dest.file_idx_of_sq() > 4 }
        }
        else if source_piece.type_of() == PieceType::P && ((source.rank_idx_of_sq() == 6 && dest.rank_idx_of_sq() == 7) || (source.rank_idx_of_sq() == 1 && dest.rank_idx_of_sq() == 0)) {
            move_type = MoveFlag::Promotion { capture: dest_piece.type_of().is_some(), prom: PieceType::Q };
        }
        else if source_piece.type_of() == PieceType::P && distance > 1 {
            if source.file_idx_of_sq() == dest.file_idx_of_sq() {
                move_type = MoveFlag::DoublePawnPush;
            }
            else if dest_piece.type_of().is_none() {
                move_type = MoveFlag::Capture { ep_capture: true };
            }
        }
        else if dest_piece.type_of().is_some() {
            move_type = MoveFlag::Capture { ep_capture: false };
        }
        
        let proposed = BitMove::init(PreMoveInfo {
            src: source,
            dst: dest,
            flags: move_type,
        });

        if board.legal_move(proposed) {
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
