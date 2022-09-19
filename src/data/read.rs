use std::{thread, time::Duration};

use crate::utils::{ConversationData, TweetData, TweetReferenceData, UserData};

use super::entities::prelude::*;
use super::entities::*;
use futures::future::join_all;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

pub async fn tweets(db: &DatabaseConnection) -> Vec<TweetData> {
    let tweet_models: Vec<tweets::Model> = Tweets::find()
        .all(db as &DatabaseConnection)
        .await
        .unwrap_or_else(|error| panic!("Failed to get tweets from database. Error: {:?}", error));
    join_all(
        tweet_models
            .into_iter()
            .map(|tweet_model| TweetData::read_from_data_model(db, tweet_model)),
    )
    .await
}

pub async fn conversation_ids(db: &DatabaseConnection) -> Vec<i64> {
    let conversation_models: Vec<conversations::Model> = Conversations::find()
        .all(db as &DatabaseConnection)
        .await
        .unwrap_or_else(|error| panic!("Failed to get conversations from database. Error: {:?}", error));
        conversation_models
            .into_iter()
            .map(|conversation_model| conversation_model.id)
            .collect()
}

pub async fn conversations(db: &DatabaseConnection) -> Vec<ConversationData> {
    let conversation_models: Vec<conversations::Model> = Conversations::find()
        .all(db as &DatabaseConnection)
        .await
        .unwrap_or_else(|error| panic!("Failed to get tweets from database. Error: {:?}", error));
    join_all(
        conversation_models
            .into_iter()
            .map(|conversation_model| conversation(db, conversation_model.id)),
    )
    .await
}

pub async fn conversation(db: &DatabaseConnection, conversation_id: i64) -> ConversationData {
    println!("Getting conversation {}", conversation_id);
    let conversation_tweets_from_db = match Tweets::find()
        .filter(tweets::Column::ConversationId.eq(conversation_id))
        .order_by_asc(tweets::Column::CreatedAt)
        .all(db)
        .await
    {
        Ok(conversation_tweets_from_db) => conversation_tweets_from_db,
        Err(error) => {
            println!("{:?}", error);
            println!("Failed to get connection... waiting 2 minutes...");
            thread::sleep(Duration::from_secs(120));
            println!("Retrying...");
            match Tweets::find()
                .filter(tweets::Column::ConversationId.eq(conversation_id))
                .order_by_asc(tweets::Column::CreatedAt)
                .all(db)
                .await
            {
                Ok(conversation_tweets_from_db) => {
                    println!("{:?}", conversation_tweets_from_db);
                    conversation_tweets_from_db
                }
                Err(error) => {
                    panic!(
                        "Failed to get conversations from database. Error: {:?}",
                        error,
                    )
                }
            }
        }
    };
    let tweets = join_all(
        conversation_tweets_from_db
            .into_iter()
            .map(|tweet_model| TweetData::read_from_data_model(db, tweet_model)),
    )
    .await;
    ConversationData {
        id: conversation_id,
        tweets,
    }
}

pub async fn conversations_given_tweets(
    db: &DatabaseConnection,
    tweets: Vec<TweetData>,
) -> Vec<ConversationData> {
    println!("first base");
    let conversation_models: Vec<conversations::Model> = Conversations::find()
        .all(db as &DatabaseConnection)
        .await
        .unwrap_or_else(|error| panic!("Failed to get tweets from database. Error: {:?}", error));
    println!("second base");
    join_all(
        conversation_models.into_iter().map(|conversation_model| {
            conversation_given_tweets(conversation_model.id, tweets.clone())
        }),
    )
    .await
}

pub async fn conversation_given_tweets(
    conversation_id: i64,
    tweets: Vec<TweetData>,
) -> ConversationData {
    println!("Getting conversation {}", conversation_id);
    let tweets = tweets
        .into_iter()
        .filter(|tweet| match tweet.clone().tweet {
            Some(tweet) => tweet.conversation_id == conversation_id,
            None => false,
        })
        .collect();
    ConversationData {
        id: conversation_id,
        tweets,
    }
}
pub async fn users(db: &DatabaseConnection) -> Vec<UserData> {
    let users_from_db = Users::find()
        .all(db)
        .await
        .unwrap_or_else(|error| panic!("Failed to get users from database. Error: {:?}", error));

    join_all(
        users_from_db
            .into_iter()
            .map(|user| UserData::from_data_model(user)),
    )
    .await
}
