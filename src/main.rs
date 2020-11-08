use std::{
    env,
    error::Error,
    process,
    result::Result,
    time::{Duration, Instant},
    thread,
};

use futures_util::{
    pin_mut,
    stream::StreamExt,
};
use mr_splashy_pants::pants::Pants;
use regex::Regex;
use rusqlite::{NO_PARAMS, Connection, params};
use signal_hook::{iterator::Signals, SIGTERM, SIGINT, SIGQUIT, SIGHUP};
use teloxide::prelude::Request;

//https://docs.rs/mr_splashy_pants/0.1.32/mr_splashy_pants/api/generated/response/listing/subreddit_new/index.html

#[derive(Debug)]
struct SeenPost {
    id: i32,
    link: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let signals = Signals::new(&[SIGTERM, SIGINT, SIGQUIT, SIGHUP])?;
    thread::spawn(move || {
        for sig in signals.forever() {
            process::exit(sig);
        }
    });

    thread::spawn(move || {
        let five_minutes = Duration::from_secs(60*5);

        loop {
            thread::sleep(five_minutes);
            println!("[heartbeat] {:?}", Instant::now())
        }
    });

    let db_path = create_db()?;

    // telegram
    let chat_id = env::var("BOT_CHAT_ID")
        .expect("BOT_CHAT_ID is not set")
        .parse::<i64>()?;
    let tg_bot = teloxide::Bot::from_env();

    // reddit
    let mut pants = create_pants();

    let subreddit = env::var("BOT_SUBREDDIT")?;
    let stream = pants.subreddit(&subreddit).stream_new();
    pin_mut!(stream);

    let (desired_keywords, undesired_keywords) = parse_keywords()?;
    println!("[parsed] desired keywords {:?}", &desired_keywords);

    if undesired_keywords.is_some() {
        println!("[parsed] NOT desired keywords {:?}", &undesired_keywords);
    }

    println!("[init] ok, listening now");
    while let Some(value) = stream.next().await {
        let value = match value {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[stream] got err: {:?}", e);
                continue;
            },
        };

        let permalink = match &value.permalink {
            Some(v) => v,
            None => continue,
        };

        if get_by_permalink(&db_path, &permalink)? {
            continue;
        } else {
            insert_by_permalink(&db_path, &permalink)?;
        }

        if let Some(title) = &value.title {
            if let Some(ref undesired) = undesired_keywords {
                if let Some(keyword) = matches_keywords(&undesired, title) {
                    println!("[dud] matched undesired keyword: {}", keyword);
                    continue;
                }
            }

            if let Some(keyword) = matches_keywords(&desired_keywords, title) {
                println!("[hit!] matched: {}", keyword);
                let msg = format!("{}\n{:?}", title, value.url);
                tg_bot.send_message(chat_id, &msg).send().await?;
            }
        }
    }

    Ok(())
}

fn create_db() -> Result<String, Box<dyn Error>> {
    let db_path = env::var("BOT_DB_PATH")
        .unwrap_or_else(|_| String::from("./sub_watcher.db"));

    let conn = Connection::open(&db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS seen_posts (
            id INTEGER PRIMARY KEY,
            link TEXT NOT NULL UNIQUE
        )",
        NO_PARAMS
    )?;

    Ok(db_path)
}

fn insert_by_permalink(db_path: &str, permalink: &str) -> Result<(), Box<dyn Error>> {
    let conn = Connection::open(&db_path)?;

    conn.execute("INSERT INTO seen_posts (link)
                  VALUES (?1)",
                  params![permalink])?;

    Ok(())
}

fn get_by_permalink(db_path: &str, permalink: &str) -> Result<bool, Box<dyn Error>> {
    let conn = Connection::open(&db_path)?;

    let mut stmt = conn.prepare(
        "SELECT * FROM seen_posts
         WHERE link = ?1"
    )?;
    let posts: Vec<SeenPost> = stmt.query_map(params![permalink], |row| {
        Ok(SeenPost {
            id: row.get(0)?,
            link: row.get(1)?,
        })
    })?
    .filter_map(Result::ok)
    .collect();

    if !posts.is_empty() {
        Ok(true)
    } else {
        Ok(false)
    }
}

fn format_title(title: &str) -> String {
    let re = Regex::new("[^a-zA-Z0-9]").expect("Unable to create regex");
    let replaced = re.replace_all(title, "").into_owned();

    replaced.to_ascii_lowercase()
}

fn matches_keywords(keywords: &[String], title: &str) -> Option<String> {
    let title = format_title(title);

    for keyword in keywords {
        if title.contains(keyword) {
            return Some(String::from(keyword))
        }
    }

    None
}

fn parse_keywords() -> Result<(Vec<String>, Option<Vec<String>>), Box<dyn Error>> {
    let var = env::var("BOT_DESIRED_KEYWORDS")?;
    let desired_keywords: Vec<String> = var.split(',')
        .map(String::from)
        .collect();

    let undesired_keywords = match env::var("BOT_UNDESIRED_KEYWORDS") {
        Ok(input) => {
            Some(input.split(',')
                .map(String::from)
                .collect())
        },
        Err(_) => None,
    };

    Ok((desired_keywords, undesired_keywords))
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

    Pants::new(&user_agent, access_token,
        &refresh_token, &client_id, &client_secret)
}
