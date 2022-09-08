use crate::{
    utils::{ConversationData, TweetData, TweetReferenceData, UserData},
};

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

pub async fn conversation(
    db: &DatabaseConnection,
    conversation_id: i64,
) -> ConversationData {
    let conversation_tweets_from_db = Tweets::find()
        .filter(tweets::Column::ConversationId.eq(conversation_id))
        .order_by_asc(tweets::Column::CreatedAt)
        .all(db)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "Failed to get conversations from database. Error: {:?}",
                error,
            )
        });
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
