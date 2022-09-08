// src/setup.rs

use sea_orm::*;

// Replace with your database URL
const DATABASE_URL: &str = "sqlite:./tweets.db";

pub(crate) async fn set_up_db() -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(DATABASE_URL.to_owned());
    opt.sqlx_logging(false);
    let db = Database::connect(opt).await?;

    // Replace with your desired database name
    let db_name = "tweets_db";
    let db = match db.get_database_backend() {
        DbBackend::MySql => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name),
            ))
            .await?;

            let url = format!("{}/{}", DATABASE_URL, db_name);
            Database::connect(&url).await?
        }
        DbBackend::Postgres => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
            ))
            .await?;
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE \"{}\";", db_name),
            ))
            .await?;

            let url = format!("{}/{}", DATABASE_URL, db_name);
            Database::connect(&url).await?
        }
        DbBackend::Sqlite => db,
    };

    Ok(db)
}


