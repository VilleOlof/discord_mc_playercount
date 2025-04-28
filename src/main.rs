use std::fs::read_to_string;
use std::time::Duration;

use elytra_ping::PingError;
use serde::{Deserialize, Serialize};
use serenity::all::{ActivityData, ChannelId};
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

#[derive(Debug, Serialize)]
struct ChangeChannelName {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    token: String,
    channel_id: u64,
    ip: String,
    port: u16,
    interval: u64,
}
impl Config {
    const PATH: &'static str = "config.toml";
}

async fn get_player_count(
    ip: String,
    timeout: Duration,
    port: u16,
) -> Result<(u32, u32), PingError> {
    let (ping_info, _) = elytra_ping::ping_or_timeout((ip, port), timeout).await?;

    let player_info = ping_info.players.unwrap();
    Ok((player_info.online, player_info.max))
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        ctx.set_activity(Some(ActivityData::playing("Create For Fan")));

        let config: Config = toml::from_str(&read_to_string(Config::PATH).unwrap()).unwrap();

        let ping_timeout = Duration::from_secs(20);

        loop {
            let name = match get_player_count(config.ip.clone(), ping_timeout, config.port).await {
                Ok((online, max)) => format!("Spelare online: {online}/{max}"),
                Err(err) => {
                    println!("Failed to ping: {err}");
                    "🔴 Servern är nere...".into()
                }
            };

            match ctx
                .http
                .edit_channel(
                    ChannelId::new(config.channel_id),
                    &ChangeChannelName { name },
                    None,
                )
                .await
            {
                Err(why) => {
                    println!("Failed to edit channel name: {why}")
                }
                Ok(_) => (),
            };

            tokio::time::sleep(Duration::from_secs(config.interval)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    let config: Config = toml::from_str(&read_to_string(Config::PATH).unwrap()).unwrap();
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&config.token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
