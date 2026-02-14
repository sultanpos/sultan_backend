#![allow(dead_code)]
pub mod branch;
pub mod category;
pub mod customer;
pub mod number;
pub mod product;
pub mod sell_price;
pub mod stock;
pub mod supplier;
pub mod token;
pub mod unit;
pub mod user;

use crate::{domain::model::pagination::PaginationOptions, snowflake::SnowflakeGenerator};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

pub static ID_GENERATOR: Lazy<Mutex<SnowflakeGenerator>> =
    Lazy::new(|| Mutex::new(SnowflakeGenerator::new(1).unwrap()));

pub async fn generate_test_id() -> i64 {
    let generator = ID_GENERATOR.lock().await;
    generator.generate().unwrap()
}

pub fn default_pagination() -> PaginationOptions {
    PaginationOptions::new(1, 100, None)
}
