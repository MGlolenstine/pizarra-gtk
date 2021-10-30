use std::env;
use std::path::{Path, PathBuf};
use std::fs::{File, create_dir_all};
use std::io::{Read, Write};

use directories::ProjectDirs;
use pizarra::config::Config;

/// Tries as hard as possible to read the current configuration. Retrieving
/// the path to it from the environment or common locations.
pub fn read() -> Config {
    if let Ok(value) = env::var("PIZARRA_CONFIG") {
        let config_path = PathBuf::from(value);

        return if config_path.is_file() {
            read_from_toml(config_path)
        } else {
            create_and_return_config(&config_path)
        };
    }

    // Next try from some known directories
    if let Some(project_dirs) = ProjectDirs::from("tk", "categulario", "pizarra") {
        let config_filename = {
            let mut conf = project_dirs.config_dir().to_owned();
            conf.push("config.toml");
            conf
        };

        if config_filename.is_file() {
            read_from_toml(config_filename)
        } else {
            let config_dir = project_dirs.config_dir();

            create_dir_all(config_dir).unwrap();
            create_and_return_config(&config_filename)
        }
    } else {
        panic!("Could not determine project dirs for your platform")
    }
}

fn read_from_toml<P: AsRef<Path>>(path: P) -> Config {
    let path: PathBuf = path.as_ref().into();

    let mut contents = String::new();
    let mut file = File::open(&path).unwrap();

    file.read_to_string(&mut contents).unwrap();

    toml::from_str(&contents).unwrap()
}

/// Assume the configuration file does not exist, create a default one and
/// return it.
fn create_and_return_config(config_filename: &Path) -> Config {
    let config = Config::default();
    let mut config_file = File::create(config_filename).unwrap();

    config_file.write_all(toml::to_string(&config).unwrap().as_bytes()).unwrap();

    config
}
