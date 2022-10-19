use crate::utils::{strip_punctuation, UserData};
use data::entities::users;
use futures::executor::block_on;
use std::fs;
use std::io::Write;
use utils::{ConversationData, TweetData, TweetWithTagsData};
mod data;
mod utils;

#[tokio::main]
async fn main() {
    println!("Reading Database");
    let db = data::setup::set_up_db()
        .await
        .expect("Failed to load database");
    println!("Database loaded");
    println!("Reading Users from Database");
    let users: Vec<UserData> = data::read::users(&db).await;
    println!("Users loaded");
    println!("Reading Tweets from Database");
    let tweets: Vec<TweetData> = data::read::tweets(&db).await;

    let corpus_string = strip_punctuation(
        tweets
            .clone()
            .into_iter()
            .flat_map(|tweet| -> Option<String> { Some(tweet.clone().tweet?.content) })
            .collect::<String>(),
    )
    .to_ascii_lowercase();
    let tweets: Vec<TweetWithTagsData> = tweets
        .iter()
        .take(50)
        .enumerate()
        .map(|(position, tweet)| {
            println!("");
            TweetWithTagsData::new(tweet.clone(), &corpus_string)
        })
        .collect();
    println!("Tweets loaded");
    println!("Reading Conversations from Database");
    let conversations: Vec<i64> = data::read::conversation_ids(&db).await;
    println!("Conversations loaded");
    println!("Creating Directories");
    create_dirs();
    println!("Directories created");
    println!("Writing Users to Markdown");
    users.iter().for_each(|user| {
        write_user(user);
    });
    println!("Users written to Markdown");
    println!("Writing Tweets to Markdown");
    tweets.iter().for_each(|tweet| {
        write_tweet(&tweet, &users);
        write_tweet_to_calendar(&tweet.tweet, &users);
    });
    let tweet_and_conversation_ids = tweets
        .iter()
        .filter_map(|tweet| match tweet.tweet.tweet {
            Some(ref tweet) => Some((tweet.id, tweet.conversation_id)),
            None => None,
        })
        .collect::<Vec<(i64, i64)>>();
    println!("Tweets written to Markdown");
    println!("Writing Conversations to Markdown");
    conversations.iter().for_each(|conversation| {
        let conversation_tweets =
            get_conversation_tweet_ids(conversation, &tweet_and_conversation_ids);
        write_conversation_from_id(conversation, &conversation_tweets);
    });
    println!("Conversations written to Markdown");
    println!("Done!");
}

fn write_conversation_from_id(conversation_id: &i64, conversation_tweets: &Vec<i64>) {
    let formatted_conversation =
        format_conversation(conversation_id.to_string(), conversation_tweets);
    let path = format!("tweet-vault/conversations/conversation-{conversation_id}.md");
    fs::write(path, formatted_conversation).expect("Failed to write conversation");
}

fn format_conversation(conversation_id: String, conversation_tweets: &Vec<i64>) -> String {
    let conversation_template = fs::read_to_string("templates/Conversation.md")
        .expect("Failed to read conversation template");
    let conversation_tweets_formatted = format_conversation_tweet(conversation_tweets);
    conversation_template
        .replace("{{CONVERSATION_ID}}", &conversation_id)
        .replace("{{CONVERSATION_TWEETS}}", &conversation_tweets_formatted)
}

fn format_conversation_tweet(conversation_tweets: &Vec<i64>) -> String {
    let mut formatted_conversation_tweets = String::from("");
    conversation_tweets.iter().for_each(|tweet_id| {
        formatted_conversation_tweets.push_str(&format!("![[{tweet_id}]]\n"));
    });
    formatted_conversation_tweets
}

fn get_conversation_tweet_ids(conversation_id: &i64, tweet_ids: &[(i64, i64)]) -> Vec<i64> {
    tweet_ids
        .iter()
        .filter_map(|(tweet_id, conversation)| {
            if conversation == conversation_id {
                Some(*tweet_id)
            } else {
                None
            }
        })
        .collect()
}

fn write_user(user_data: &UserData) {
    let user = user_data.clone().user.expect("Invalid user");
    let path = format!("tweet-vault/users/@{}.md", user.username);
    let formatted_user = format_user(user_data);
    fs::write(path, formatted_user).expect("Unable to write file");
}

fn format_user(user_data: &UserData) -> String {
    let user = user_data.clone().user.expect("Invalid user");
    let user_template =
        fs::read_to_string("templates/User.md").expect("Unable to read user template");
    user_template
        .replace("{{USER_ID}}", &user.id.to_string())
        .replace("{{USER_FULL_NAME}}", &user.name)
        .replace("{{TWITTER_HANDLE}}", &format!("@{}", &user.username))
        .replace("{{USER_DESCRIPTION}}", &user.description)
}

fn write_tweet(tweet_data: &TweetWithTagsData, users: &[UserData]) {
    let tweet = tweet_data
        .clone()
        .tweet
        .clone()
        .tweet
        .expect("Invalid tweet");
    let tags = tweet_data.tags.clone();
    let formatted_tweet = format_tweet(&tweet_data.tweet, tags, users);
    let path = format!("tweet-vault/tweets/{}.md", tweet.id);
    fs::write(path, formatted_tweet).expect("Unable to write file");
}

fn format_tweet(tweet_data: &TweetData, tags: Vec<String>, users: &[UserData]) -> String {
    let tweet = tweet_data.clone().tweet.expect("Invalid tweet");
    let user = users
        .iter()
        .find(|user| user.user.clone().expect("Invalid user").id == tweet.author_id)
        .expect("Invalid user")
        .user
        .clone()
        .unwrap();
    let author_twitter_handle = user.username;
    let tweet_template =
        fs::read_to_string("templates/Tweet.md").expect("Failed to read tweet template");
    tweet_template
        .replace("{{CONTENT}}", &tweet.content)
        .replace("{{TWEET_ID}}", &tweet.id.to_string())
        .replace("{{AUTHOR_TWITTER_HANDLE}}", &author_twitter_handle)
        .replace("{{PUBLISHED_DATE}}", &tweet.created_at.to_string())
        .replace("{{CONVERSATION_ID}}", &tweet.conversation_id.to_string())
        .replace("{{IN_REPLY_TO_ID}}", &format_in_reply_to(tweet_data))
        .replace("{{RETWEET_ID}}", &format_retweeted(tweet_data))
        .replace("{{QUOTED_TWEET_ID}}", &format_quoted(tweet_data))
        .replace("{{TAGS}}", &format_tags(&tags))
}

fn format_in_reply_to(tweet_data: &TweetData) -> String {
    let references = tweet_data.clone().references;
    let reply_to = references
        .into_iter()
        .find(|reference| reference.reference_type == "replied_to");
    match reply_to {
        Some(reply_to) => {
            format!("{}", reply_to.referenced_tweet_id)
        }
        None => String::from("None"),
    }
}

fn format_retweeted(tweet_data: &TweetData) -> String {
    let references = tweet_data.clone().references;
    let reply_to = references
        .into_iter()
        .find(|reference| reference.reference_type == "retweeted");
    match reply_to {
        Some(reply_to) => {
            format!("{}", reply_to.referenced_tweet_id)
        }
        None => String::from("None"),
    }
}

fn format_quoted(tweet_data: &TweetData) -> String {
    let references = tweet_data.clone().references;
    let reply_to = references
        .into_iter()
        .find(|reference| reference.reference_type == "quoted");
    match reply_to {
        Some(reply_to) => {
            format!("{}", reply_to.referenced_tweet_id)
        }
        None => String::from("None"),
    }
}

fn format_tags(tags: &Vec<String>) -> String {
    tags.into_iter().fold("".to_string(), |output, tag| {
        output + &format!("- {}\n", tag)
    })
}

fn write_tweet_to_calendar(tweet_data: &TweetData, users: &[UserData]) {
    let tweet = tweet_data.clone().tweet.expect("Invalid tweet");
    let embedded_tweet_string = format!("![[{}]]\n", tweet.id);
    let date = tweet.created_at.date_naive().to_string();
    let path = format!("tweet-vault/calendar/{}.md", date);
    let mut file = fs::File::options()
        .create(true)
        .write(true)
        .append(true)
        .open(path)
        .expect("Failed to open or create file");
    write!(file, "{}", embedded_tweet_string).expect("Failed to write file");
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
    match fs::create_dir("tweet-vault/calendar") {
        Ok(_) => println!("Created tweet-vault/calendar directory"),
        Err(_) => println!("tweet-vault/calendar directory already exists"),
    };
}
