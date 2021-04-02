use crossterm::event;
use directories::ProjectDirs;
use std::path::PathBuf;
use tini::Ini;

#[derive(Debug)]
pub struct Config {
    ini: Ini,
    path: PathBuf,
    pub view_comments: event::KeyCode,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Config {
            ini: Ini::new().section("keybindings").item("view_comments", 'C'),
            path: Self::config_path(),
            view_comments: event::KeyCode::Char('C'),
        }
    }
}

impl Config {
    pub fn new() -> Result<Self, std::io::Error> {
        let mut config = Self::default();

        if let Some(parent) = config.path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        if !config.path.exists() {
            config.write()?;
        } else {
            if let Err(err) = config.read() {
                eprintln!("{:#?}, {:#?}", err, config.path);
            }
        }

        println!("{:#?}", config);

        Ok(config)
    }

    pub fn read(&mut self) -> Result<(), tini::Error> {
        self.ini = Ini::from_file(&self.path)?;
        for (name, section_iter) in self.ini.iter() {
            match name.as_str() {
                "keybindings" => {
                    for (key, value) in section_iter {
                        match key.as_str() {
                            "view_comments" => {
                                self.view_comments =
                                    event::KeyCode::Char(value.chars().next().expect("Invalid key"))
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn write(&self) -> Result<(), std::io::Error> {
        self.ini.to_file(self.path.as_path())?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        let mut config_dir = ProjectDirs::from("me", "bramw", "hntui")
            .unwrap()
            .config_dir()
            .to_path_buf();
        config_dir.push("config.yml");
        config_dir
    }
}
