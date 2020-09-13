# rust-reddit-watcher-telegram-notifier

Watches a specific subreddit for keywords.

If a match is found, it sends the `title` & `url` of the reddit post to a specified telegram channel.

### env vars

Telegram:

```
BOT_CHAT_ID=
TELOXIDE_TOKEN=
```

Reddit:

* [how to get reddit tokens](https://github.com/reddit-archive/reddit/wiki/OAuth2)

```
BOT_KEYWORDS=p30ls,vp9
BOT_SUBREDDIT=gundeals

BOT_USER_AGENT=
BOT_ACCESS_TOKEN=
BOT_REFRESH_TOKEN=
BOT_CLIENT_ID=
BOT_CLIENT_SECRET=
```
