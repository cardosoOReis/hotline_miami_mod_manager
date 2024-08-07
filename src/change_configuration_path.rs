use std::path::PathBuf;

use inquire::{error::InquireResult, Select};

use crate::{
    configs::paths_config::{ConfigurationPath, WithPath, WithoutPath},
    functions::get_user_input,
};

pub fn get_desired_path_to_change() -> InquireResult<ConfigurationPath<WithPath>> {
    let variants = Vec::from(ConfigurationPath::VARIANTS);
    let configuration_path =
        Select::new("What folder's path do you want to change?", variants).prompt()?;
    validate_path(configuration_path)
}

fn validate_path(
    configuration_path: ConfigurationPath<WithoutPath>,
) -> InquireResult<ConfigurationPath<WithPath>> {
    let path = get_user_input(
        &format!("The path for your {} folder:", configuration_path.name())
    );
    if std::fs::metadata(&path).is_ok() {
        Ok(configuration_path.with_path(PathBuf::from(path)))
    } else {
        println!("Couldn't validate this path: {}. Please try again.", &path);
        validate_path(configuration_path)
    }
}
