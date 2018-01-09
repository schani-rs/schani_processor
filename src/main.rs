extern crate dotenv;
extern crate env_logger;
extern crate schani_processor;

use dotenv::dotenv;

fn main() {
    env_logger::init().unwrap();
    dotenv().ok();

    schani_processor::run();
}
