use futures::executor::block_on;

use crate::utils::UserData;

mod utils;
mod data;

fn main() {
    let db = block_on(data::setup::set_up_db()).expect("Failed to load database");
    let users:Vec<UserData> = block_on(data::read::users(&db));
    users.iter().for_each(|user|println!("{:?}", user.user));
}
