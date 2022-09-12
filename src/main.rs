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
//    users.iter().for_each(|user| println!("{:?}", user.user));
    
    create_dirs();
    tweets.into_iter().for_each(|tweet| {
        write_tweet(tweet);
    });

    
}

fn write_tweet(tweet_data: TweetData) {
    let formatted_tweet = format_tweet(&tweet_data);
    let tweet = tweet_data.tweet.expect("Invalid tweet");
    let path = format!("tweet-vault/tweets/{}.md", tweet.id);
    fs::write(path, formatted_tweet).expect("Unable to write file");
}

fn format_tweet(tweet_data: &TweetData)-> String {
    let tweet = tweet_data.clone().tweet.expect("Invalid tweet");
    let tweet_template = fs::read_to_string("templates/Tweet.md").expect("Failed to read tweet template");
        tweet_template
            .replace("{{CONTENT}}", &tweet.content)
            .replace("{{TWEET_ID}}", &tweet.id.to_string())
            .replace("{{AUTHOR_TWITTER_HANDLE}}", &tweet.author_id.to_string())
            .replace("{{PUBLISHED_DATE}}", &tweet.created_at.to_string())
            .replace("{{CONVERSATION_ID}}", &tweet.conversation_id.to_string())
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
