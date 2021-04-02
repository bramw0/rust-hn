mod config;
mod items;
mod ui;

/*
 * Config library -> specify max items in config, specify default view (Top vs New) in config
 * Display note to user when items are being collected (requested) for the New view or Top view
 * Add C keybinding to open comments in browser
 * Customizable keybindings?
 */

use api::Post;
use crossterm::{
    event, event::Event as CEvent, terminal::disable_raw_mode, terminal::enable_raw_mode,
};
use lazy_static::lazy_static;
use regex::Regex;
use std::str;
use std::{sync::mpsc, thread, time::Duration, time::Instant};

use config::Config;
use items::{NewItems, TopItems};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols::DOT,
    terminal::Terminal,
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Tabs},
};
use ui::{MenuItem, StatefulList};
use webbrowser;

enum Event<I> {
    Input(I),
    Tick,
}

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r".+//(?P<url>[^/]*)").unwrap();
}

fn setup_input() -> mpsc::Receiver<Event<event::KeyEvent>> {
    let (sender, receiver) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    thread::spawn(move || {
        let mut last_tick = Instant::now();

        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).is_ok() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    sender.send(Event::Input(key)).unwrap();
                }
            }

            if last_tick.elapsed() >= tick_rate && sender.send(Event::Tick).is_ok() {
                last_tick = Instant::now();
            }
        }
    });

    receiver
}

fn extract_url(url: &str) -> Option<&str> {
    match URL_REGEX.captures(url) {
        Some(capture) => {
            if let Some(url) = capture.name("url") {
                Some(url.as_str())
            } else {
                None
            }
        }
        None => None,
    }
}

fn generate_list_items(items: Vec<(usize, Post)>) -> Vec<ListItem<'static>> {
    let list_items = {
        items
            .iter()
            .map(|(pos, post)| match post {
                Post::Comment(comment) => ListItem::new(Spans::from(vec![
                    Span::styled(format!("{}", pos), Style::default().fg(Color::Red)),
                    Span::raw(format!(" {:?}", comment)),
                ])),
                Post::Job(job) => {
                    let url = match extract_url(job.url.as_str()) {
                        Some(uri) => uri,
                        None => job.url.as_str(),
                    };

                    ListItem::new(Spans::from(vec![
                        Span::styled(format!("{}", pos), Style::default().fg(Color::Red)),
                        Span::styled(format!(" {}", job.title.clone()), Style::default()),
                        Span::styled(
                            format!(" ({})", url),
                            Style::default()
                                .fg(Color::Gray)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    ]))
                }
                Post::Poll(poll) => ListItem::new(Spans::from(vec![
                    Span::styled(format!("{}", pos), Style::default().fg(Color::Red)),
                    Span::raw(format!(" {}", poll.title.clone())),
                ])),
                Post::PollOpt(poll_opt) => ListItem::new(Spans::from(vec![
                    Span::styled(format!("{}", pos), Style::default().fg(Color::Red)),
                    Span::raw(format!(" {:?}", poll_opt)),
                ])),
                Post::Story(story) => {
                    let url = match extract_url(story.url.as_str()) {
                        Some(uri) => uri,
                        None => story.url.as_str(),
                    };

                    ListItem::new(Spans::from(vec![
                        Span::styled(format!("{}", pos), Style::default().fg(Color::Red)),
                        Span::styled(format!(" {}", story.title.clone()), Style::default()),
                        Span::styled(
                            format!(" ({})", url),
                            Style::default()
                                .fg(Color::Gray)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    ]))
                }
            })
            .collect()
    };

    list_items
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client: api::Client = api::Client::new(api::BASE_URL.to_string(), ureq::Agent::new());
    let mut top_items: TopItems = TopItems::new(client.clone());
    let mut new_items: NewItems = NewItems::new(client.clone());
    let mut config = Config::new()?;

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    enable_raw_mode()?;

    let receiver = setup_input();

    top_items.list = generate_list_items(top_items.get_vec()?);
    let mut stateful_list = StatefulList::new(top_items.list.clone());
    stateful_list.next();

    let mut active_menu_item = MenuItem::Top;
    let tabs: Vec<Spans> = ["Top", "New"].iter().cloned().map(Spans::from).collect();

    loop {
        terminal.draw(|frame| match active_menu_item {
            MenuItem::Top => {
                if stateful_list.items != top_items.list {
                    stateful_list.items =
                        generate_list_items(top_items.get_vec().unwrap_or_else(|_| vec![]));
                }

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
                    .split(frame.size());

                let list = List::new(stateful_list.items.clone())
                    .block(
                        Block::default().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT),
                    )
                    .highlight_style(Style::default().fg(Color::Green));

                let tab = Tabs::new(tabs.clone())
                    .select(active_menu_item.into())
                    .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT))
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .divider(DOT);

                frame.render_widget(tab, chunks[0]);
                frame.render_stateful_widget(list, chunks[1], &mut stateful_list.state);
            }
            MenuItem::New => {
                if stateful_list.items != new_items.list {
                    stateful_list.items =
                        generate_list_items(new_items.get_vec().unwrap_or_else(|_| vec![]));
                }

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
                    .split(frame.size());

                let list = List::new(stateful_list.items.clone())
                    .block(
                        Block::default().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT),
                    )
                    .highlight_style(Style::default().fg(Color::Green));

                let tab = Tabs::new(tabs.clone())
                    .select(active_menu_item.into())
                    .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT))
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .divider(DOT);

                frame.render_widget(tab, chunks[0]);
                frame.render_stateful_widget(list, chunks[1], &mut stateful_list.state);
            }
        })?;

        match receiver.recv()? {
            Event::Input(event) => match event.code {
                event::KeyCode::Char('q') | event::KeyCode::Esc => {
                    disable_raw_mode()?;
                    terminal.clear()?;
                    terminal.show_cursor()?;
                    break;
                }
                event::KeyCode::Char('j') | event::KeyCode::Down => {
                    stateful_list.next();
                }
                event::KeyCode::Char('k') | event::KeyCode::Up => {
                    stateful_list.previous();
                }
                event::KeyCode::Char('h')
                | event::KeyCode::Left
                | event::KeyCode::Char('l')
                | event::KeyCode::Right => {
                    active_menu_item.scroll();
                }
                event::KeyCode::Enter => {
                    if let Some(index) = stateful_list.state.selected() {
                        let items = match active_menu_item {
                            MenuItem::Top => {
                                let top_items = &mut top_items;
                                top_items.get_vec()?
                            }
                            MenuItem::New => {
                                let new_items = &mut new_items;
                                new_items.get_vec()?
                            }
                        };

                        if let Some(item) = items.get(index) {
                            match item {
                                (_, Post::Comment(comment)) => eprintln!("{:?}", comment),
                                (_, Post::Job(job)) => match webbrowser::open(job.url.as_str()) {
                                    Ok(_) => {
                                        terminal.clear().expect("Failed to clear the terminal");
                                    }
                                    Err(error) => {
                                        eprintln!("{:?}", error);
                                    }
                                },
                                (_, Post::Poll(poll)) => eprintln!("{:?}", poll),
                                (_, Post::PollOpt(poll_opt)) => eprintln!("{:?}", poll_opt),
                                (_, Post::Story(story)) => {
                                    match webbrowser::open(story.url.as_str()) {
                                        Ok(_) => {
                                            terminal.clear().expect("Failed to clear the terminal")
                                        }
                                        Err(error) => {
                                            eprintln!("{:?}", error);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    if event.code == config.view_comments {
                        if let Some(index) = stateful_list.state.selected() {
                            let items = match active_menu_item {
                                MenuItem::Top => {
                                    let top_items = &mut top_items;
                                    top_items.get_vec()?
                                }
                                MenuItem::New => {
                                    let new_items = &mut new_items;
                                    new_items.get_vec()?
                                }
                            };

                            if let Some(item) = items.get(index) {
                                match item {
                                    (_, Post::Comment(comment)) => eprintln!("{:?}", comment),
                                    (_, Post::Job(job)) => match webbrowser::open(&format!(
                                        "https://news.ycombinator.com/item?id={}",
                                        job.id
                                    )) {
                                        Ok(_) => {
                                            terminal.clear().expect("Failed to clear the terminal");
                                        }
                                        Err(error) => {
                                            eprintln!("{:?}", error);
                                        }
                                    },
                                    (_, Post::Poll(poll)) => eprintln!("{:?}", poll),
                                    (_, Post::PollOpt(poll_opt)) => eprintln!("{:?}", poll_opt),
                                    (_, Post::Story(story)) => {
                                        match webbrowser::open(&format!(
                                            "https://news.ycombinator.com/item?id={}",
                                            story.id
                                        )) {
                                            Ok(_) => terminal
                                                .clear()
                                                .expect("Failed to clear the terminal"),
                                            Err(error) => {
                                                eprintln!("{:?}", error);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Event::Tick => {}
        }
    }

    Ok(())
}
