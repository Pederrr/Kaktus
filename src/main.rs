use std::fs::{read_to_string, write};
use std::process::exit;

use kaktus::scrape::{get_kaktus_page, get_latest_message};

const FILE_PATH: &str = "last_message";

// TODO use cron to run the script every few hours
// Maybe use tmpfs to make the file stay in ram,
// therefore making the SD card last longer
//
// maybe loop and sleep for hour might be better for the card

fn main() {
    let Ok(old_message) = read_to_string(FILE_PATH) else {
        // TODO the bot itself can send this
        eprintln!("Unable to read the file containg the old message");
        exit(1)
    };

    let Ok(page) = get_kaktus_page() else {
        // TODO the bot itself can send this
        eprintln!("Unable to download the page");
        exit(1)
    };
    let Some(new_message) = get_latest_message(&page) else {
        // TODO the bot itself can send this
        eprintln!("Unable to fetch the latest message");
        exit(1)
    };

    if old_message != new_message {
        println!("We got a new message!");
        if write(FILE_PATH, new_message).is_err() {
            // TODO the bot itself can send this
            eprintln!("Unable to write the new message to the file");
            exit(1)
        };
    }
}
