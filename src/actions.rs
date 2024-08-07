use std::fmt::Display;


#[derive(Debug, Clone, Copy)]
pub enum Action {
    ChangeMod,
    RunGame,
    UseDefaultSettings,
    CreateNewModFolder,
    ChangeConfigurationPath,
    ClearConfiguration,
    Exit,
}

impl Action {
    pub const VARIANTS: &'static [Action] = &[
        Action::ChangeMod,
        Action::RunGame,
        Action::UseDefaultSettings,
        Action::CreateNewModFolder,
        Action::ChangeConfigurationPath,
        Action::ClearConfiguration,
        Action::Exit,
    ];
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::ChangeMod => write!(f, "Change the current mod."),
            Action::RunGame => write!(f, "Run Hotline Miami 2."),
            Action::UseDefaultSettings => write!(f, "Use the default setting's (Normal game music without mods)."),
            Action::CreateNewModFolder => write!(f, "Create a new mod folder structure."),
            Action::ChangeConfigurationPath => write!(f, "Change one of your paths."),
            Action::ClearConfiguration => write!(f, "Clear your configuration."),
            Action::Exit => write!(f, "Exit."),
        }
    }
}
