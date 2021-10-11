use regex::Regex;
use std::fs;
use std::path::PathBuf;

const ALACRITTY_CONFIG: &'static str = "alacritty.yml";
const ALACRITTY_COLOR_SCHEMES: &'static str = "alacritty_color_schemes.yml";

#[cfg(not(test))]
const ALACRITTY_CONFIG_PATHS: [&'static str; 8] = [
    "$XDG_CONFIG_HOME/alacritty/alacritty.yml",
    "$XDG_CONFIG_HOME/alacritty.yml",
    "$HOME/.config/alacritty/alacritty.yml",
    "$HOME/.alacritty.yml",
    "$XDG_CONFIG_HOME/alacritty/alacritty_color_schemes.yml",
    "$XDG_CONFIG_HOME/alacritty_color_schemes.yml",
    "$HOME/.config/alacritty/alacritty_color_schemes.yml",
    "$HOME/.alacritty_color_schemes.yml",
];

#[cfg(not(test))]
const HOME_SHELL_VAR: &'static str = "HOME";

#[cfg(test)]
pub const ALACRITTY_CONFIG_PATHS: [&'static str; 2] = [
    "$TEST/assets/sample_alacritty.yml",
    "$TEST/assets/sample_alacritty_color_schemes.yml",
];

#[cfg(test)]
const HOME_SHELL_VAR: &'static str = "TEST";

pub struct AlacrittyConfig {
    pub path: PathBuf,
    pub data: String,
}
pub struct AlacrittyColorSchemes {
    pub path: PathBuf,
    pub data: String
}
pub struct ColorScheme {
    name: String,
    data: String
}

#[derive(Debug)]
pub enum Error {
    MissingAlacrittyYaml,
    MissingColorSchemes,
    InvalidColorScheme(String),
    ReadHomeVarError,
    DuplicateConfigFiles,
    CurrentSchemeError
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingAlacrittyYaml | Error::MissingColorSchemes => {
                write!(f, "Ensure your config and/or colorschemes are at one of the following: {}", alacritty_config_paths())
            },
            Error::ReadHomeVarError => write!(f, "Unable to read $HOME or $XDG_CONFIG_HOME shell variables."),
            Error::DuplicateConfigFiles => write!(f, "Looks like you may have duplicate alacritty.yml and/or alacritty_color_schemes.yml files."),
            Error::CurrentSchemeError => write!(f, "Could not ascertain current color scheme."),
            Error::InvalidColorScheme(e) => write!(f, "Could not find scheme '{}' in alacritty_color_schemes.yml.", e)
        }
    }
}

impl ColorScheme {
    fn new(name: String, data: String) -> Self {
        ColorScheme { name, data }
    }
}

impl AlacrittyConfig {
    fn new(path: PathBuf) -> Result<Self, std::io::Error>  {
        let data = fs::read_to_string(&path)?;

        Ok(Self { path, data })
    }

    pub fn get_current_scheme_name(&self) -> Result<String, Error> {
        let re = Regex::new(r"\bscheme:\s{1}&[A-Za-z0-9_]+").unwrap();

        if let Some(m) = re.find(&self.data) {
            Ok(String::from(m.as_str().to_string().trim_start_matches("scheme: &")))
        } else {
            Err(Error::CurrentSchemeError)
        }
    }

    pub fn set_scheme(&mut self, scheme: &ColorScheme) {
        let re_scheme = Regex::new(r"\bscheme:.*(?:\n\s{2,}.+)+").unwrap();
        let re_name = Regex::new(&format!(r"{}:", &scheme.name)).unwrap();
        let re_color = Regex::new(r"\bcolors:.*").unwrap();

        let mut data = re_name.replace(&scheme.data, "scheme:");
        data = re_scheme.replace(&self.data, &data);

        let new_config = re_color.replace(&data, format!("colors: *{}", &scheme.name));

        self.data = new_config.to_string();
    }

    pub fn apply_scheme(&self) -> Result<(), std::io::Error> {
        fs::write(&self.path, &self.data)?;

        Ok(())
    }
}

impl AlacrittyColorSchemes {
    fn new(path: PathBuf) -> Result<Self, std::io::Error>  {
        let data = fs::read_to_string(&path)?;

        Ok(Self{ path, data })
    }

    pub fn get_available_schemes(&self) -> Result<Vec<String>, std::io::Error> {
        let re = Regex::new(r"&[A-Za-z0-9_-]+").unwrap();

        let colors: Vec<String> = re
            .find_iter(&self.data)
            .map(|color| color.as_str().trim_start_matches('&').to_string())
            .collect();

        Ok(colors)
    }

    pub fn get_scheme(&self, name: &str) -> Result<ColorScheme, Error>{
        let re = Regex::new(&format!(r"{}:.*(?:\n\s{{4,}}.+)+", name)).unwrap();

        let scheme = match re.find(&self.data) {
            Some(s) => s,
            None => return Err(Error::InvalidColorScheme(name.to_string()))
        };

        let cs = ColorScheme::new(name.to_string(), String::from(scheme.as_str()));

        Ok(cs)
    }
}

pub fn find_alacritty_configs() -> Result<(AlacrittyConfig, AlacrittyColorSchemes), Box<dyn std::error::Error>> {
    let homedir = if let Ok(h) = std::env::var(HOME_SHELL_VAR) {
        h
    } else if let Ok(h) = std::env::var("XDG_CONFIG_HOME") {
        h
    } else {
        return Err(Box::new(Error::ReadHomeVarError))
    };

    let re_home = Regex::new(r"^\$[A-Z_]+").unwrap();

    let paths: Vec<String> = ALACRITTY_CONFIG_PATHS
        .iter()
        .map(|path| re_home.replace(path, &homedir).into_owned())
        .filter(|path| if let Ok(_) = fs::metadata(&path) { true } else { false })
        .collect();

    if paths.len() > 2 {
        return Err(Box::new(Error::DuplicateConfigFiles))
    }

    let mut color_schemes_path = PathBuf::new();
    if let Some(p) = paths.iter().find(|p| p.contains(ALACRITTY_COLOR_SCHEMES)) {
        color_schemes_path.push(&p)
    } else {
        return Err(Box::new(Error::MissingColorSchemes))
    }

    let mut config_path = PathBuf::new();
    if let Some(p) = paths.iter().find(|p| p.contains(ALACRITTY_CONFIG)) {
        config_path.push(&p)
    } else {
        return Err(Box::new(Error::MissingAlacrittyYaml))
    }

    let config = match AlacrittyConfig::new(config_path) {
        Ok(c) => c,
        Err(e) => return Err(Box::new(e))
    };

    let schemes = match AlacrittyColorSchemes::new(color_schemes_path) {
        Ok(s) => s,
        Err(e) => return Err(Box::new(e))
    };

    Ok((config, schemes))
}

fn alacritty_config_paths() -> String {
    let mut paths = "".to_string();

    for path in ALACRITTY_CONFIG_PATHS {
        paths.push_str(&format!("{}\n", path))
    }
    paths
}
