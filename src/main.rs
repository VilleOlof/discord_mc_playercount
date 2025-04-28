use std::fs::read_to_string;
use std::time::Duration;

use elytra_ping::PingError;
use serde::{Deserialize, Serialize};
use serenity::all::{ActivityData, ChannelId};
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

#[derive(Debug, Deserialize)]
struct ConfigDiscord {
    token: String,
    channel_id: u64,
}
#[derive(Debug, Deserialize)]
struct ConfigMinecraft {
    ip: String,
    port: u16,
    interval: u64,
}
#[derive(Debug, Deserialize)]
struct ConfigFormat {
    online: String,
    offline: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    discord: ConfigDiscord,
    minecraft: ConfigMinecraft,
    format: ConfigFormat,
}
impl Config {
    const PATH: &'static str = "config.toml";
}

#[derive(Debug, Serialize)]
struct ChangeChannelName {
    name: String,
}

async fn get_player_count(
    ip: String,
    timeout: Duration,
    port: u16,
) -> Result<(u32, u32), PingError> {
    let (ping_info, latency) = elytra_ping::ping_or_timeout((ip, port), timeout).await?;

    println!("Took {latency:?} to ping server");

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
            let name = match get_player_count(
                config.minecraft.ip.clone(),
                ping_timeout,
                config.minecraft.port,
            )
            .await
            {
                Ok((online, max)) => config
                    .format
                    .online
                    .replace("$ONLINE", &online.to_string())
                    .replace("$MAX", &max.to_string()),
                Err(err) => {
                    println!("Failed to ping: {err:?}");
                    config.format.offline.clone()
                }
            };

            match ctx
                .http
                .edit_channel(
                    ChannelId::new(config.discord.channel_id),
                    &ChangeChannelName { name },
                    None,
                )
                .await
            {
                Err(why) => {
                    println!("Failed to edit channel name: {why:?}")
                }
                Ok(_) => (),
            };

            tokio::time::sleep(Duration::from_secs(config.minecraft.interval)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    let config: Config = toml::from_str(&read_to_string(Config::PATH).unwrap()).unwrap();
    let intents = GatewayIntents::empty();

    let mut client = Client::builder(&config.discord.token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
