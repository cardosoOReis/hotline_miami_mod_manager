use crate::{
    change_configuration_path::get_desired_path_to_change,
    configs::paths_config::ConfigurationPath, exit::exit,
};
use std::{fs, ops::Deref};

use anyhow::{anyhow, bail};
use inquire::{InquireError, Select};

use crate::{
    actions::Action,
    configs::{
        paths_config::{ModsGroupPath, ProgramPath},
        Configs,
    },
    create_new_mod_folder::create_new_mod_folder,
    hotline_mod::{HotlineMod, Music},
    replace_mod::{replace_mods::replace_mods, replace_music::replace_music},
    select_mod::{select_mod, ChangeCurrentModError},
};

pub struct HotlineModManager {
    default_game: Option<DefaultHotlineMod>,
    all_mods: AllMods,
    configs: Configs,
}

impl HotlineModManager {
    pub fn build() -> anyhow::Result<Self> {
        let configs = Configs::build()?;
        let mut all_mods = list_mods(configs.paths_config().mods_group_path())?;
        let default_game_index = all_mods.0.iter().position(|hm_mod| {
            hm_mod.name().directory_name().to_string_lossy() == "hotline_miami_2"
        });

        let default_game = default_game_index
            .map(|index| all_mods.0.remove(index))
            .map(DefaultHotlineMod);

        if default_game.is_none() {
            println!("{ORIGINAL_GAME_SETTINGS_NOT_FOUND_WARNING}");
        }

        Ok(HotlineModManager {
            default_game,
            all_mods,
            configs,
        })
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            let action = self.get_action();
            if let Err(error) = match action {
                Action::ChangeMod => self.change_mod(),
                Action::RunGame => Self::run_hotline_miami_2(),
                Action::UseDefaultSettings => self.use_default_settings(),
                Action::CreateNewModFolder => self.create_new_mod_folder(),
                Action::ChangeConfigurationPath => self.change_configuration_path(),
                Action::ClearConfiguration => self.clear_configuration(),
                Action::Exit => exit(),
            } {
                println!("{error}");
            }
        }
    }

    fn get_action(&self) -> Action {
        self.print_mod_name();
        let prompt =
            Select::new("What do you want to do?", Action::VARIANTS.iter().collect()).prompt();

        match prompt {
            Ok(action) => *action,
            Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => {
                panic!("User asked to quit the program")
            }
            Err(error) => panic!("{}", error),
        }
    }

    fn change_mod(&mut self) -> anyhow::Result<()> {
        let desired_mod = match select_mod(&self.all_mods.0) {
            Ok(desired_mod) => desired_mod,
            Err(ChangeCurrentModError::EmptyMods) => {
                println!("You have no mods in your folder right now. Try downloading new mods or bringing your existing mods to this folder.");
                return Ok(());
            }
            Err(ChangeCurrentModError::UserCanceledOperation) => return Ok(()),
            Err(ChangeCurrentModError::UserExitedApplication) => {
                panic!("User requested to leave the application.")
            }
            Err(ChangeCurrentModError::InquireError(err)) => return Err(anyhow!(err)),
        };

        let default_game_music = self.default_game_music();

        let music = desired_mod.music().or(default_game_music);

        if let Some(music) = music {
            replace_music(
                self.configs.paths_config().game_path(),
                music,
                desired_mod.name(),
            )?;
        } else {
            println!("{ORIGINAL_GAME_SETTINGS_NOT_FOUND_WARNING}");
        };

        replace_mods(
            self.configs.paths_config().mods_path(),
            desired_mod.mods(),
            desired_mod.name(),
        )?;

        self.configs.set_current_mod(desired_mod.name().clone())?;

        Ok(())
    }

    fn default_game_music(&self) -> Option<&Music> {
        self.default_game
            .as_ref()
            .and_then(|default_mod| default_mod.music())
    }

    fn run_hotline_miami_2() -> anyhow::Result<()> {
        if let Err(err) = crate::run_hotline_miami_2() {
            Err(anyhow!(err))
        } else {
            std::process::exit(0)
        }
    }

    fn use_default_settings(&mut self) -> anyhow::Result<()> {
        match &self.default_game {
            None => {
                println!("{ORIGINAL_GAME_SETTINGS_NOT_FOUND_WARNING}");
                Ok(())
            }
            Some(DefaultHotlineMod(hm_mod)) if hm_mod.music().is_none() => {
                println!("{ORIGINAL_GAME_SETTINGS_NOT_FOUND_WARNING}");
                Ok(())
            }
            Some(DefaultHotlineMod(hm_mod)) => {
                replace_music(
                    self.configs.paths_config().game_path(),
                    hm_mod.music().expect("Music validation in above branch"),
                    hm_mod.name(),
                )?;
                replace_mods(
                    self.configs.paths_config().mods_path(),
                    hm_mod.mods(),
                    hm_mod.name(),
                )?;

                self.configs.set_current_mod(hm_mod.name().clone())?;

                Ok(())
            }
        }
    }

    fn clear_configuration(&mut self) -> anyhow::Result<()> {
        self.configs.clear()?;

        self.configs = Configs::build()?;

        Ok(())
    }

    fn create_new_mod_folder(&mut self) -> anyhow::Result<()> {
        let new_mod = create_new_mod_folder(
            &self.all_mods,
            self.configs.paths_config().mods_group_path(),
        )?;
        self.all_mods.0.push(HotlineMod::from_name(new_mod));

        Ok(())
    }

    fn change_configuration_path(&mut self) -> anyhow::Result<()> {
        match get_desired_path_to_change() {
            Ok(ConfigurationPath::Game(path)) => {
                self.configs.mut_paths_config().set_game_path(path.into());
            }
            Ok(ConfigurationPath::Mods(path)) => {
                self.configs.mut_paths_config().set_mods_path(path.into());
            }
            Ok(ConfigurationPath::Group(path)) => {
                self.configs
                    .mut_paths_config()
                    .set_mods_group_path(path.into());
            }
            Err(InquireError::OperationCanceled) => return Ok(()),
            Err(InquireError::OperationInterrupted) => panic!("User requested to quit application"),
            Err(err) => bail!(err),
        };

        Ok(())
    }
    
    fn print_mod_name(&self) {
        let mod_name = self.configs.current_mod()
            .map(|current_mod| current_mod.name().formatted_name())
            .map_or("Uncertain...", AsRef::as_ref);
    
        println!("You are currently using: {mod_name}");
    }
}


pub struct DefaultHotlineMod(HotlineMod);

impl DefaultHotlineMod {
    pub fn hm_mod(&self) -> &HotlineMod {
        self
    }
}

impl Deref for DefaultHotlineMod {
    type Target = HotlineMod;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn list_mods(mods_path: &ModsGroupPath) -> anyhow::Result<AllMods> {
    let vec = fs::read_dir(mods_path.path())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter_map(|path| HotlineMod::new(&path))
        .collect();

    Ok(AllMods::new(vec))
}

pub struct AllMods(Vec<HotlineMod>);

impl AllMods {
    fn new(mods: Vec<HotlineMod>) -> Self {
        AllMods(mods)
    }

    pub fn mods(&self) -> &[HotlineMod] {
        &self.0
    }

    pub fn mods_mut(&mut self) -> &mut Vec<HotlineMod> {
        &mut self.0
    }
}

const ORIGINAL_GAME_SETTINGS_NOT_FOUND_WARNING: &str = "Attention: You don't have a folder for the original Hotline Miami 2 with the correct configuration. It should contain the original game's music and be named 'hotline_miami_2'. Without it, the program may not behave as expected. For more information, visit the project's GitHub (https://github.com/cardosoOReis/hotline_miami_mod_manager.git).";
