use dirs;
use failure::err_msg;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use termion::event::Key;

const FILE_NAME: &str = "config.yml";
const CONFIG_DIR: &str = ".config";
const APP_CONFIG_DIR: &str = "spotify-tui";

fn parse_key(key: String) -> Result<Key, failure::Error> {
    fn get_single_char(string: &str) -> char {
        match string.chars().nth(0) {
            Some(c) => c,
            None => panic!(),
        }
    }

    match key.len() {
        1 => Ok(Key::Char(get_single_char(key.as_str()))),
        _ => {
            let sections: Vec<&str> = key.split('-').collect();

            if sections.len() > 2 {
                return Err(failure::format_err!(
                    "Shortcut can only have 2 keys, \"{}\" has {}",
                    key,
                    sections.len()
                ));
            }

            match sections[0].to_lowercase().as_str() {
                "ctrl" => Ok(Key::Ctrl(get_single_char(sections[1]))),
                "alt" => Ok(Key::Alt(get_single_char(sections[1]))),
                "left" => Ok(Key::Left),
                "right" => Ok(Key::Right),
                "up" => Ok(Key::Up),
                "down" => Ok(Key::Down),
                "backspace" | "delete" => Ok(Key::Backspace),
                "del" => Ok(Key::Delete),
                "esc" | "escape" => Ok(Key::Esc),
                "pageup" => Ok(Key::PageUp),
                "pagedown" => Ok(Key::PageDown),
                "space" => Ok(Key::Char(' ')),
                _ => Err(failure::format_err!(
                    "The key \"{}\" is unknown.",
                    sections[0]
                )),
            }
        }
    }
}

fn check_reserved_keys(key: Key) -> Result<(), failure::Error> {
    let reserved = [
        Key::Char('h'),
        Key::Char('j'),
        Key::Char('k'),
        Key::Char('l'),
        Key::Up,
        Key::Down,
        Key::Left,
        Key::Right,
        Key::Backspace,
        Key::Char('\n'),
    ];
    for item in reserved.iter() {
        if key == *item {
            // TODO: Add pretty print for key
            return Err(failure::format_err!(
                "The key {:?} is reserved and cannot be remapped",
                key
            ));
        }
    }
    Ok(())
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserConfigString {
    back: Option<String>,
    jump_to_album: Option<String>,
    jump_to_artist_album: Option<String>,
    manage_devices: Option<String>,
    decrease_volume: Option<String>,
    increase_volume: Option<String>,
    toggle_playback: Option<String>,
    seek_backwards: Option<String>,
    seek_forwards: Option<String>,
    next_track: Option<String>,
    previous_track: Option<String>,
    help: Option<String>,
    shuffle: Option<String>,
    repeat: Option<String>,
    search: Option<String>,
}

pub struct UserConfig {
    pub back: Key,
    pub jump_to_album: Key,
    pub jump_to_artist_album: Key,
    pub manage_devices: Key,
    pub decrease_volume: Key,
    pub increase_volume: Key,
    pub toggle_playback: Key,
    pub seek_backwards: Key,
    pub seek_forwards: Key,
    pub next_track: Key,
    pub previous_track: Key,
    pub help: Key,
    pub shuffle: Key,
    pub repeat: Key,
    pub search: Key,
}

pub struct UserConfigPaths {
    pub config_file_path: PathBuf,
}

impl UserConfig {
    pub fn new() -> UserConfig {
        UserConfig {
            back: Key::Char('q'),
            jump_to_album: Key::Char('a'),
            jump_to_artist_album: Key::Char('A'),
            manage_devices: Key::Char('d'),
            decrease_volume: Key::Char('-'),
            increase_volume: Key::Char('+'),
            toggle_playback: Key::Char(' '),
            seek_backwards: Key::Char('<'),
            seek_forwards: Key::Char('>'),
            next_track: Key::Char('n'),
            previous_track: Key::Char('p'),
            help: Key::Char('?'),
            shuffle: Key::Char('s'),
            repeat: Key::Char('r'),
            search: Key::Char('/'),
        }
    }

    pub fn get_or_build_paths(&self) -> Result<(UserConfigPaths), failure::Error> {
        match dirs::home_dir() {
            Some(home) => {
                let path = Path::new(&home);
                let home_config_dir = path.join(CONFIG_DIR);
                let app_config_dir = home_config_dir.join(APP_CONFIG_DIR);

                if !home_config_dir.exists() {
                    fs::create_dir(&home_config_dir)?;
                }

                if !app_config_dir.exists() {
                    fs::create_dir(&app_config_dir)?;
                }

                let config_file_path = &app_config_dir.join(FILE_NAME);

                let paths = UserConfigPaths {
                    config_file_path: config_file_path.to_path_buf(),
                };

                Ok(paths)
            }
            None => Err(err_msg("No $HOME directory found for client config")),
        }
    }

    pub fn load_config(&mut self) -> Result<(), failure::Error> {
        let paths = self.get_or_build_paths()?;
        if paths.config_file_path.exists() {
            let config_string = fs::read_to_string(&paths.config_file_path)?;
            // serde fails if file is empty
            if config_string.trim().is_empty() {
                return Ok(());
            }
            let config_yml: UserConfigString = serde_yaml::from_str(&config_string)?;

            macro_rules! to_keys {
                ($name: ident) => {
                    if let Some(key_string) = config_yml.$name {
                        self.$name = parse_key(key_string)?;
                        check_reserved_keys(self.$name)?;
                    }
                };
            };

            to_keys!(back);
            to_keys!(jump_to_album);
            to_keys!(jump_to_artist_album);
            to_keys!(manage_devices);
            to_keys!(decrease_volume);
            to_keys!(increase_volume);
            to_keys!(toggle_playback);
            to_keys!(seek_backwards);
            to_keys!(seek_forwards);
            to_keys!(next_track);
            to_keys!(previous_track);
            to_keys!(help);
            to_keys!(shuffle);
            to_keys!(repeat);
            to_keys!(search);

            Ok(())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_key() {
        use super::parse_key;
        use termion::event::Key;
        assert_eq!(parse_key(String::from("j")).unwrap(), Key::Char('j'));
        assert_eq!(parse_key(String::from("J")).unwrap(), Key::Char('J'));
        assert_eq!(parse_key(String::from("ctrl-j")).unwrap(), Key::Ctrl('j'));
        assert_eq!(parse_key(String::from("ctrl-J")).unwrap(), Key::Ctrl('J'));
        assert_eq!(parse_key(String::from("-")).unwrap(), Key::Char('-'));
        assert_eq!(parse_key(String::from("esc")).unwrap(), Key::Esc);
        assert_eq!(parse_key(String::from("del")).unwrap(), Key::Delete);
    }

    #[test]
    fn test_reserved_key() {
        use super::check_reserved_keys;
        use termion::event::Key;

        assert!(
            check_reserved_keys(&Key::Char('\n')).is_err(),
            "Enter key should be reserved"
        );
    }
}
