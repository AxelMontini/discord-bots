use anyhow::Context;
use chrono::{DateTime, Duration, Utc};
use once_cell::sync::OnceCell;
use rand::prelude::*;
use regex::Regex;
use serenity::{
    async_trait,
    model::{
        channel::{Channel, Message},
        id::ChannelId,
    },
    prelude::*,
    utils::MessageBuilder,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use structopt::StructOpt;
use utils::SortedVec;

static WORD_REGEX: OnceCell<Regex> = OnceCell::new();

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Options {
    /// The discord token to use
    #[structopt(long)]
    pub token: String,
    /// Min interval between messages
    #[structopt(long, default_value = "600")]
    pub interval_low: u64,
    /// Max interval between bessages
    #[structopt(long, default_value = "1200")]
    pub interval_high: u64,
    /// Words are separated by a whitespace
    #[structopt(long, default_value = "^[a-zA-ZàáèéìíòóùúÀÁÈÉÌÍÒÓÙÚ']+$")]
    pub word_regex: String,
    /// Instances of words older than this are deleted to save space and forget dead memes.
    #[structopt(long, default_value = "1800")]
    pub max_age: u64,
    /// Max random boost to word count. If set to 3, a word said 8 times might be texted even if there's a word texted 10 times.
    #[structopt(long, default_value = "10")]
    pub max_boost: usize,
    /// If no words have been said, the bot will print this word as default. Leave blank to not print anything by default.
    #[structopt(long)]
    pub default_word: Option<String>,
}

type WordMap = HashMap<String, SortedVec<DateTime<Utc>>>;

struct MessageMap;

impl TypeMapKey for MessageMap {
    type Value = Arc<RwLock<WordMap>>;
}

struct RecentChannel;

impl TypeMapKey for RecentChannel {
    type Value = Arc<RwLock<Option<ChannelId>>>;
}

struct Reader;

#[async_trait]
impl EventHandler for Reader {
    async fn message(&self, context: serenity::client::Context, msg: Message) {
        // skip if own message
        if msg.author.id == context.http.get_current_user().await.unwrap().id {
            return; // do nothing if we sent the message
        }

        let regex = WORD_REGEX.get().unwrap();

        // iterate over words defined by the regex
        let word_iterator = msg
            .content
            .split_whitespace()
            .filter(|word| regex.is_match(word))
            .map(|word| word.to_lowercase());

        {
            let data_read = context.data.read().await;
            let recent_channel_lock = data_read
                .get::<RecentChannel>()
                .expect("RecentChannel to be in context")
                .clone();

            // Set most current channel. Pino will reply there.
            recent_channel_lock.write().unwrap().replace(msg.channel_id);
        }

        let message_map_lock = {
            let data_read = context.data.read().await;
            data_read
                .get::<MessageMap>()
                .expect("MessageMap to be in context")
                .clone()
        };

        let mut message_map = message_map_lock.write().unwrap();

        let time = msg.timestamp;

        for word in word_iterator {
            if let Some(value) = message_map.get_mut(&word) {
                value.insert(time);
            } else {
                message_map.insert(word, SortedVec::from_vec(vec![time]));
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = Options::from_args();

    println!("Starting PinoBot 🦜");

    WORD_REGEX
        .set(Regex::new(&options.word_regex).context("compiling regex")?)
        .unwrap();

    let mut client = Client::builder(&options.token)
        .event_handler(Reader)
        .await
        .expect("creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<MessageMap>(Arc::new(RwLock::new(HashMap::new())));
        data.insert::<RecentChannel>(Arc::new(RwLock::new(None)));
    }

    let cache_and_http = client.cache_and_http.clone();
    let data = client.data.clone();

    tokio::spawn(async move {
        let mut rng = rand::rngs::StdRng::seed_from_u64(69);

        loop {
            let time: u64 = rng.gen_range(options.interval_low..=options.interval_high);

            println!("Sending message in {} seconds", time);

            tokio::time::delay_for(Duration::seconds(time as i64).to_std().unwrap()).await;

            // Send message
            let data_read = data.read().await;

            let mut boost = || rng.gen_range(0..=options.max_boost);

            let maybe_word = {
                let words = data_read.get::<MessageMap>().unwrap().read().unwrap();
                let maybe_word = words
                    .iter()
                    .max_by_key(|(_word, instances)| instances.len() + boost())
                    .map(|(word, _)| word.to_owned());

                maybe_word.or(options.default_word.clone())
            };

            if let Some(word) = maybe_word {
                let recent_channel = data_read
                    .get::<RecentChannel>()
                    .expect("RecentChannel to be in data/context");

                let locked_channel = *recent_channel.read().expect("locking recent channel");

                if let Some(channel) = locked_channel.clone() {
                    let message = MessageBuilder::new().push(&word).build();

                    if let Err(e) = channel.clone().say(&cache_and_http.http, message).await {
                        println!("Error sending message: {}", e);
                    } else {
                        println!("Send message '{}' to channel '{:?}' 🦜", word, channel);
                    }
                } else {
                    println!("Most recent channel is None, type some text to update it!");
                }

                // Clean up old words
                let older_than = Utc::now() - Duration::seconds(options.max_age as i64);

                let mut words = data_read.get::<MessageMap>().unwrap().write().unwrap();
                // Remove words older than older_than
                for val in words.values_mut() {
                    val.remove_le(&older_than);
                }
                // Remove entries with empty vectors to save space
                words.retain(|_k, vec| vec.len() != 0);
            }
        }
    });

    client.start().await.context("starting client")
}
