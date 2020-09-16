use std::{
    env,
    error::Error,
    process,
    result::Result,
    thread,
};

use futures_util::{
    pin_mut,
    stream::StreamExt,
};
use mr_splashy_pants::Pants;
use regex::Regex;
use signal_hook::{iterator::Signals, SIGTERM, SIGINT, SIGQUIT, SIGHUP};
use teloxide::prelude::Request;

// https://docs.rs/mr_splashy_pants/0.1.18/src/mr_splashy_pants/api/generated/response/listing/subreddit_new.rs.html

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let signals = Signals::new(&[SIGTERM, SIGINT, SIGQUIT, SIGHUP])?;
    thread::spawn(move || {
        for sig in signals.forever() {
            process::exit(sig);
        }
    });

    // telegram
    let chat_id = env::var("BOT_CHAT_ID")
        .expect("BOT_CHAT_ID is not set")
        .parse::<i64>()?;
    let tg_bot = teloxide::Bot::from_env();

    // reddit
    let mut pants = create_pants();

    let subreddit = env::var("BOT_SUBREDDIT")?;
    let stream = pants.stream_subreddit_new(&subreddit);
    pin_mut!(stream);

    let keywords = parse_keywords()?;
    println!("[parsed] {:?}", keywords);

    println!("[init] ok, listening now");
    while let Some(value) = stream.next().await {
        if matches_keywords(&keywords, &value.title) {
            let msg = format!("{}\n{}", value.title, value.url);
            tg_bot.send_message(chat_id, &msg).send().await?;
        }
    }

    Ok(())
}

fn format_title(title: &str) -> String {
    let re = Regex::new("[^a-zA-Z0-9]").expect("Unable to create regex");
    let replaced = re.replace_all(title, "").into_owned();

    replaced.to_ascii_lowercase()
}

fn matches_keywords(keywords: &[String], title: &str) -> bool {
    let title = format_title(title);

    for keyword in keywords {
        if title.contains(keyword) {
            println!("[match] {}: {}", &keyword, &title);
            return true
        }
    }

    false
}

fn parse_keywords() -> Result<Vec<String>, Box<dyn Error>> {
    let var = env::var("BOT_KEYWORDS")?;

    let keywords: Vec<String> = var.split(',')
        .map(String::from)
        .collect();

    Ok(keywords)
}

fn create_pants() -> Pants {
    let user_agent = env::var("BOT_USER_AGENT")
        .expect("BOT_USER_AGENT is not set");
    let access_token = env::var("BOT_ACCESS_TOKEN")
        .expect("BOT_ACCESS_TOKEN is not set");
    let refresh_token = env::var("BOT_REFRESH_TOKEN")
        .expect("BOT_REFRESH_TOKEN is not set");
    let client_id = env::var("BOT_CLIENT_ID")
        .expect("BOT_CLIENT_ID is not set");
    let client_secret = env::var("BOT_CLIENT_SECRET")
        .expect("BOT_CLIENT_SECRET is not set");

    Pants::new(&user_agent, &access_token,
        refresh_token, &client_id, &client_secret)
}
