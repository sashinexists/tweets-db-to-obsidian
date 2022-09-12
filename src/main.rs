use data::entities::users;
use futures::executor::block_on;
use utils::TweetData;

use crate::utils::UserData;
use std::fs;
mod data;
mod utils;

fn main() {
    let db = block_on(data::setup::set_up_db()).expect("Failed to load database");
    let users: Vec<UserData> = block_on(data::read::users(&db));
    let tweets: Vec<TweetData> = block_on(data::read::tweets(&db));
    create_dirs();
    tweets.into_iter().for_each(|tweet| {
        write_tweet(tweet, &users);
    });

    
}

fn write_tweet(tweet_data: TweetData, users: &[UserData]) {
    let tweet = tweet_data.clone().tweet.expect("Invalid tweet");
    let formatted_tweet = format_tweet(&tweet_data, users);
    let path = format!("tweet-vault/tweets/{}.md", tweet.id);
    fs::write(path, formatted_tweet).expect("Unable to write file");
}

fn format_tweet(tweet_data: &TweetData, users: &[UserData])-> String {
    let tweet = tweet_data.clone().tweet.expect("Invalid tweet");
    let user = users.iter().find(|user| user.user.clone().expect("Invalid user").id == tweet.author_id).expect("Invalid user").user.clone().unwrap();
    let author_twitter_handle = user.username;
    let tweet_template = fs::read_to_string("templates/Tweet.md").expect("Failed to read tweet template");
        tweet_template
            .replace("{{CONTENT}}", &tweet.content)
            .replace("{{TWEET_ID}}", &tweet.id.to_string())
            .replace("{{AUTHOR_TWITTER_HANDLE}}", &format!("[[@{}]]",&author_twitter_handle))
            .replace("{{PUBLISHED_DATE}}", &tweet.created_at.to_string())
            .replace("{{CONVERSATION_ID}}", &format!("[[Conversation-{}]]",&tweet.conversation_id))
            .replace("{{IN_REPLY_TO_ID}}", &format_in_reply_to(tweet_data))
            .replace("{{RETWEET_ID}}", &format_retweeted(tweet_data))
            .replace("{{QUOTED_TWEET_ID}}", &format_quoted(tweet_data))
}

fn format_in_reply_to(tweet_data: &TweetData) -> String {
    let references = tweet_data.clone().references;
    let reply_to = references.into_iter().find(|reference| reference.reference_type == "replied_to");
    match reply_to {
        Some(reply_to) => {
            format!("[[{}]]", reply_to.referenced_tweet_id)
        },
        None => String::from("None"),
    }
}


fn format_retweeted(tweet_data: &TweetData) -> String {
    let references = tweet_data.clone().references;
    let reply_to = references.into_iter().find(|reference| reference.reference_type == "retweeted");
    match reply_to {
        Some(reply_to) => {
            format!("[[{}]]", reply_to.referenced_tweet_id)
        },
        None => String::from("None"),
    }
}

fn format_quoted(tweet_data: &TweetData) -> String {
    let references = tweet_data.clone().references;
    let reply_to = references.into_iter().find(|reference| reference.reference_type == "quoted");
    match reply_to {
        Some(reply_to) => {
            format!("[[{}]]", reply_to.referenced_tweet_id)
        },
        None => String::from("None"),
    }
}
fn create_dirs() {
    match fs::create_dir("tweet-vault") {
        Ok(_) => println!("Created tweet-vault directory"),
        Err(_) => println!("tweet-vault directory already exists"),
    };
    match fs::create_dir("tweet-vault/tweets") {
        Ok(_) => println!("Created tweet-vault/tweets directory"),
        Err(_) => println!("tweet-vault/tweets directory already exists"),
    };
    match fs::create_dir("tweet-vault/users") {
        Ok(_) => println!("Created tweet-vault/users directory"),
        Err(_) => println!("tweet-vault/users directory already exists"),
    };
    match fs::create_dir("tweet-vault/conversations") {
        Ok(_) => println!("Created tweet-vault/conversations directory"),
        Err(_) => println!("tweet-vault/conversations directory already exists"),
    };
}
