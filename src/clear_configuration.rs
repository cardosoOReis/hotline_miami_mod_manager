use std::fs;

use crate::configs::CONFIGS_FILE_NAME;

pub fn clear_configuration() {
    if let Err(error) = fs::remove_file(CONFIGS_FILE_NAME) {
        println!("Error clearing the configs. {error}");
    } else {
        println!("Successfully cleared your configuration!");
    };
}
