use crate::ui::MenuItem;
use crossterm::event::KeyCode;
use directories::ProjectDirs;
use std::{collections::HashSet, path::PathBuf};
use tini::Ini;

#[derive(Debug)]
pub struct Config {
    ini: Ini,
    path: PathBuf,
    pub view_comments: HashSet<KeyCode>,
    pub quit: HashSet<KeyCode>,
    pub down: HashSet<KeyCode>,
    pub up: HashSet<KeyCode>,
    pub left: HashSet<KeyCode>,
    pub right: HashSet<KeyCode>,
    pub open_article: HashSet<KeyCode>,
    pub refresh: HashSet<KeyCode>,
    pub max_items: u16,
    pub default_view: MenuItem,
}

impl std::default::Default for Config {
    fn default() -> Self {
        let mut view_comments = HashSet::new();
        view_comments.insert(KeyCode::Char('c'));

        let mut quit = HashSet::new();
        quit.insert(KeyCode::Char('q'));
        quit.insert(KeyCode::Esc);

        let mut down = HashSet::new();
        down.insert(KeyCode::Char('j'));
        down.insert(KeyCode::Down);

        let mut up = HashSet::new();
        up.insert(KeyCode::Char('k'));
        up.insert(KeyCode::Up);

        let mut left = HashSet::new();
        left.insert(KeyCode::Char('h'));
        left.insert(KeyCode::Left);

        let mut right = HashSet::new();
        right.insert(KeyCode::Char('l'));
        right.insert(KeyCode::Right);

        let mut open_article = HashSet::new();
        open_article.insert(KeyCode::Enter);

        let mut refresh = HashSet::new();
        refresh.insert(KeyCode::Char('r'));

        Config {
            ini: Ini::new()
                .section("keybindings")
                .item_vec("view_comments", &["c"])
                .item_vec("quit", &["q", "esc"])
                .item_vec("down", &["j", "arrow_down"])
                .item_vec("up", &["k", "arrow_up"])
                .item_vec("left", &["h", "arrow_left"])
                .item_vec("right", &["l", "arrow_right"])
                .item_vec("open_article", &["enter"])
                .item_vec("refresh", &["r"])
                .section("general")
                .item("max_items", 30)
                .item("default_view", "top"),
            path: Self::config_path(),
            view_comments,
            quit,
            down,
            up,
            left,
            right,
            open_article,
            refresh,
            max_items: 30,
            default_view: MenuItem::Top,
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
        } else if let Err(err) = config.read() {
            eprintln!("{:#?}, {:#?}", err, config.path);
        }

        Ok(config)
    }

    pub fn read(&mut self) -> Result<(), tini::Error> {
        self.ini = Ini::from_file(&self.path)?;
        self.default_view = match self.ini.get::<String>("general", "default_view") {
            Some(value) => match value.to_lowercase().as_str() {
                "top" => MenuItem::Top,
                "new" => MenuItem::New,
                _ => {
                    eprintln!("{} is not a valid default_view value", value);
                    MenuItem::Top
                }
            },
            _ => MenuItem::Top,
        };

        for (name, section_iter) in self.ini.iter() {
            match name.as_str() {
                "keybindings" => {
                    for (key, value) in section_iter {
                        let shortcuts: Vec<&str> = value.split(", ").collect();
                        match key.as_str() {
                            "view_comments" => {
                                self.view_comments = Self::parse_shortcuts(shortcuts)
                            }
                            "quit" => self.quit = Self::parse_shortcuts(shortcuts),
                            "down" => self.down = Self::parse_shortcuts(shortcuts),
                            "up" => self.up = Self::parse_shortcuts(shortcuts),
                            "left" => self.left = Self::parse_shortcuts(shortcuts),
                            "right" => self.right = Self::parse_shortcuts(shortcuts),
                            "open_article" => self.open_article = Self::parse_shortcuts(shortcuts),
                            "refresh" => self.refresh = Self::parse_shortcuts(shortcuts),
                            _ => {}
                        }
                    }
                }
                "general" => {
                    for (key, value) in section_iter {
                        match key.as_str() {
                            "max_items" => {
                                let max_items = value.parse::<u16>().unwrap_or_else(|_| {
                                    panic!("{} is not a valid max_items value", value)
                                });

                                if max_items > 500 {
                                    panic!("A max_items value greater than 500 is useless as the API returns a max of 500 posts");
                                } else {
                                    self.max_items = max_items;
                                }
                            }
                            "default_view" => {
                                self.default_view = match value.to_lowercase().as_str() {
                                    "top" => MenuItem::Top,
                                    "new" => MenuItem::New,
                                    _ => {
                                        eprintln!(
                                            "{} is not a valid default_view value",
                                            value.to_lowercase()
                                        );
                                        MenuItem::Top
                                    }
                                }
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
        config_dir.push("config.ini");
        config_dir
    }

    fn parse_shortcuts(shortcuts: Vec<&str>) -> HashSet<KeyCode> {
        shortcuts
            .iter()
            .map(|shortcut| match *shortcut {
                "backspace" => KeyCode::Backspace,
                "enter" => KeyCode::Enter,
                "arrow_left" => KeyCode::Left,
                "arrow_right" => KeyCode::Right,
                "arrow_up" => KeyCode::Up,
                "arrow_down" => KeyCode::Down,
                "home" => KeyCode::Home,
                "end" => KeyCode::End,
                "page_up" => KeyCode::PageUp,
                "page_down" => KeyCode::PageDown,
                "tab" => KeyCode::Tab,
                "back_tab" => KeyCode::BackTab,
                "delete" => KeyCode::Delete,
                "insert" => KeyCode::Insert,
                "esc" => KeyCode::Esc,
                char => {
                    if char.len() != 1 {
                        if char.starts_with('f') {
                            let mut key = char.to_string();
                            key.remove(0);

                            return KeyCode::F(key.parse::<u8>().unwrap_or_else(|err| {
                                panic!("{}: {} is not a valid F key", err, key)
                            }));
                        }
                        panic!("{} is not a valid shortcut", char);
                    }
                    KeyCode::Char(char.chars().next().unwrap())
                }
            })
            .collect()
    }
}
