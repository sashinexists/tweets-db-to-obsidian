use chrono::{DateTime, FixedOffset};
use futures::{future::join_all, StreamExt};
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::data::entities::prelude::*;
use crate::data::entities::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweetData {
    pub tweet: Option<tweets::Model>,
    pub references: Vec<tweet_references::Model>,
}

impl TweetData {
    pub fn new(tweet: tweets::Model, references: Vec<tweet_references::Model>) -> Self {
        TweetData {
            tweet: Some(tweet),
            references,
        }
    }

    pub fn empty() -> Self {
        Self {
            tweet: None,
            references: Vec::new(),
        }
    }

    pub async fn read(db: &DatabaseConnection, id: i64) -> Self {
        let references = TweetReferences::find()
            .filter(tweet_references::Column::SourceTweetId.eq(id))
            .all(db)
            .await
            .unwrap_or_else(|error| {
                panic!(
                    "Failed to get tweet references for tweet of id {id}. Error: {:?}",
                    error
                )
            })
            .into_iter()
            .collect();
        let tweet = Tweets::find_by_id(id)
            .one(db)
            .await
            .unwrap_or_else(|error| {
                panic!("Failed to get tweet {id} from database. Error: {:?}", error)
            });

        Self { tweet, references }
    }

    pub async fn read_from_data_model(
        db: &DatabaseConnection,
        tweet_model: tweets::Model,
    ) -> Self {
        let references = TweetReferences::find()
            .filter(tweet_references::Column::SourceTweetId.eq(tweet_model.id))
            .all(db)
            .await
            .unwrap_or_else(|error| {
                panic!(
                    "Failed to get tweet references for tweet of id {}. Error: {:?}",
                    tweet_model.id, error
                )
            })
            .into_iter()
            .collect();
        Self {
            tweet: Some(tweet_model),
            references,
        }
    }


    pub async fn read_many(db: &DatabaseConnection, ids: &[i64]) -> Vec<Self> {
        join_all(ids.iter().map(|id| Self::read(db, *id))).await
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub user: Option<users::Model>,
}

impl UserData {

    pub async fn empty() -> Self {
        Self { user: None }
    }

    pub async fn from_data_model(user_from_db: users::Model) -> Self {
        UserData {
            user: Some(user_from_db),
        }
    }

    pub async fn read(db: &DatabaseConnection, id: i64) -> Self {
        let user = Users::find_by_id(id).one(db).await.unwrap_or_else(|error| {
            panic!("Failed to get tweet {id} from database. Error: {:?}", error)
        });

        Self { user }
    }

    pub async fn read_from_twitter_handle(
        db: &DatabaseConnection,
        twitter_handle: &str,
    ) -> Self {
        let user = Users::find()
            .filter(users::Column::Username.eq(twitter_handle))
            .one(db)
            .await
            .unwrap_or_else(|error| {
                panic!(
                    "Failed to read user @{twitter_handle} from database. Error: {:?}",
                    error
                )
            });
        Self { user }
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationData {
    pub id: i64,
    pub tweets: Vec<TweetData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferencedTweetKind {
    RepliedTo,
    Quoted,
    Retweeted
}

#[derive(Debug, Serialize)]
pub struct TweetReferenceData {
    pub reference_type: ReferencedTweetKind,
    pub source_tweet_id: i64,
    pub reference_tweet_id: i64,
}

impl TweetReferenceData {
    pub fn type_to_string(&self) -> String {
        match self.reference_type {
            ReferencedTweetKind::RepliedTo => "replied_to",
            ReferencedTweetKind::Retweeted => "retweeted",
            ReferencedTweetKind::Quoted => "quoted",
        }
        .to_string()
    }

    pub fn kind_from_string(input: &str) -> Option<ReferencedTweetKind> {
        match input {
            "replied_to" => Some(ReferencedTweetKind::RepliedTo),
            "retweeted" => Some(ReferencedTweetKind::Retweeted),
            "quoted" => Some(ReferencedTweetKind::Quoted),
            _ => None,
        }
    }


    pub fn clone(&self) -> Self {
        Self {
            reference_type: self.reference_type.clone(),
            source_tweet_id: self.source_tweet_id,
            reference_tweet_id: self.reference_tweet_id,
        }
    }
}

pub fn i64_to_u64(i: i64) -> u64 {
    i.try_into()
        .unwrap_or_else(|error| panic!("Failed to parse u64 from i64. Error:\n{error}"))
}

pub fn u64_to_i64(u: u64) -> i64 {
    u.try_into()
        .unwrap_or_else(|error| panic!("Failed to parse i64 from u64. Error:\n{error}"))
}
