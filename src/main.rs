use std::{
    collections::HashSet,
    env, process,
    sync::Arc,
    time::{Duration, SystemTime},
};

use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    model::{channel::Message, gateway::Ready, id::ChannelId},
    prelude::*,
};

use kaktus::scrape::{get_kaktus_page, get_latest_message, KaktusMessage};

struct LastMessage;

impl TypeMapKey for LastMessage {
    type Value = Arc<RwLock<KaktusMessage>>;
}

struct LastUpdate;

impl TypeMapKey for LastUpdate {
    type Value = Arc<RwLock<SystemTime>>;
}

struct RegisteredChannels;

impl TypeMapKey for RegisteredChannels {
    type Value = Arc<RwLock<HashSet<ChannelId>>>;
}

struct Handler;

const SLEEP_TIME: u64 = 3600;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.content.as_ref() {
            "!status" => {
                let last_message = {
                    let data_read = ctx.data.read().await;
                    data_read
                        .get::<LastMessage>()
                        .expect("LastMessage must be in the type map")
                        .clone()
                        .read()
                        .await
                        .clone()
                };

                let last_update_time = {
                    let data_read = ctx.data.read().await;
                    *data_read
                        .get::<LastUpdate>()
                        .expect("LastUpdate must be in the type map")
                        .clone()
                        .read()
                        .await
                };

                let time_till_next_update = Duration::from_secs(SLEEP_TIME)
                    .saturating_sub(SystemTime::now().duration_since(last_update_time).unwrap());

                let message = CreateMessage::new().embed(
                    CreateEmbed::new()
                        .title("Status update")
                        .field(last_message.header, last_message.content, false)
                        .field(
                            "next update",
                            format!("In {time_till_next_update:?}"),
                            false,
                        ),
                );

                if let Err(why) = msg.channel_id.send_message(&ctx.http, message).await {
                    eprintln!("Error sending message: {why:?}");
                    return;
                }
            }
            "!register" => {
                let register_lock = {
                    let data_read = ctx.data.read().await;
                    data_read
                        .get::<RegisteredChannels>()
                        .expect("RegisteredChannels must be in the type map")
                        .clone()
                };

                {
                    let mut registered_channels = register_lock.write().await;

                    registered_channels.insert(msg.channel_id);
                    println!("{:?}", registered_channels);
                    println!("{:?}", registered_channels);
                    if let Err(why) = msg
                        .channel_id
                        .say(&ctx.http, "This channel will get updates from now on!")
                        .await
                    {
                        eprintln!("Error sending message: {why:?}");
                        return;
                    }
                };
            }
            "!unregister" => {
                let register_lock = {
                    let data_read = ctx.data.read().await;
                    data_read
                        .get::<RegisteredChannels>()
                        .expect("RegisteredChannels must be in the type map")
                        .clone()
                };

                {
                    let mut registered_channels = register_lock.write().await;

                    registered_channels.remove(&msg.channel_id);
                    if let Err(why) = msg
                        .channel_id
                        .say(
                            &ctx.http,
                            "This channel will no longer get updates from now on!",
                        )
                        .await
                    {
                        eprintln!("Error sending message: {why:?}");
                        return;
                    }
                }
            }
            _ => {}
        }
        if msg.content == "!register" {}
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        let ctx = Arc::new(ctx);
        let ctx_copy = ctx.clone();

        tokio::spawn(async move {
            loop {
                let Ok(page) = get_kaktus_page().await else {
                    eprintln!("Unable to download the page");
                    return;
                };
                let Some(new_message) = get_latest_message(&page) else {
                    eprintln!("Unable to fetch the latest message");
                    return;
                };

                {
                    let data_read = ctx_copy.data.read().await;
                    let lock = data_read
                        .get::<LastUpdate>()
                        .expect("LastUpdate must be in the type map")
                        .clone();
                    let mut last_update = lock.write().await;
                    *last_update = SystemTime::now()
                }

                let mut changed = false;
                {
                    let data_read = ctx_copy.data.read().await;
                    let lock = data_read
                        .get::<LastMessage>()
                        .expect("LastUpdate must be in the type map")
                        .clone();
                    let mut last_message = lock.write().await;

                    if *last_message != new_message {
                        *last_message = new_message.clone();
                        changed = true;
                    }
                }

                let register_lock = {
                    let data_read = ctx.data.read().await;
                    data_read
                        .get::<RegisteredChannels>()
                        .expect("RegisteredChannels must be in the type map")
                        .clone()
                };

                let message =
                    CreateMessage::new().embed(CreateEmbed::new().title("New message!").field(
                        new_message.header,
                        new_message.content,
                        true,
                    ));

                {
                    let registered_channels = register_lock.read().await;

                    if changed {
                        for channel_id in registered_channels.iter() {
                            if let Err(why) = channel_id
                                .send_message(&ctx_copy.http, message.clone())
                                .await
                            {
                                eprintln!("Error sending message: {why:?}");
                            };
                        }
                    }
                }

                tokio::time::sleep(Duration::from_secs(SLEEP_TIME)).await;
            }
        });
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("The DISCORD_TOKEN needs to be set in ENV");
    let default_channel_id: ChannelId = env::var("DISCORD_CHANNEL_ID")
        .expect("The DISCORD_CHANNEL_ID needs to be set in ENV")
        .parse()
        .expect("Incorrent DISCORD_CHANNEL_ID");

    let intents = GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let Ok(mut client) = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
    else {
        process::exit(1);
    };

    {
        let mut data = client.data.write().await;
        data.insert::<LastMessage>(Arc::new(RwLock::new(KaktusMessage::default())));
        data.insert::<RegisteredChannels>(Arc::new(RwLock::new(HashSet::from([
            default_channel_id,
        ]))));
        data.insert::<LastUpdate>(Arc::new(RwLock::new(SystemTime::now())));
    }

    if let Err(why) = client.start().await {
        eprintln!("Client error: {why:?}");
        process::exit(1);
    }
}
