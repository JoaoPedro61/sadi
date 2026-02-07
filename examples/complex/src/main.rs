use std::any::TypeId;

use crate::infra::persistence::sqlite::SqliteClient;

pub mod core;
pub mod infra;

fn main() {
    let app = infra::di::build().expect("Failed to build application");

    println!("TypeApp? {:?}", TypeId::of::<SqliteClient>());

    println!("Application {:?}", app);

    println!("Hello, world!");
}
