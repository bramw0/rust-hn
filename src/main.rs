mod config;
mod items;
mod ui;

/*
 * // TODO: Config library -> specify max items in config, specify default view (Top vs New) in config
 * // TODO: Display note to user when items are being collected (requested) for the New view or Top view
 */

use api::Post;
use crossterm::{
    event, event::Event as CEvent, terminal::disable_raw_mode, terminal::enable_raw_mode,
};
use futures::executor::block_on;
use lazy_static::lazy_static;
use regex::Regex;
use std::str;
use std::{sync::mpsc, thread, time::Duration, time::Instant};
use time::OffsetDateTime;

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
use ui::{MenuItem, PostItem, StatefulList};

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

fn get_time_offset(current_time: OffsetDateTime, post_time: OffsetDateTime) -> String {
    let offset = current_time - post_time;

    let weeks = offset.whole_weeks();
    if weeks > 0 {
        return format!(
            "{} {} ago",
            weeks,
            if weeks == 1 { "week" } else { "weeks" }
        );
    } else {
        let days = offset.whole_days();
        if days > 0 {
            return format!("{} {} ago", days, if days == 1 { "day" } else { "days" });
        } else {
            let hours = offset.whole_hours();
            if hours > 0 {
                return format!(
                    "{} {} ago",
                    hours,
                    if hours == 1 { "hour" } else { "hours" }
                );
            } else {
                let mins = offset.whole_minutes();
                if mins > 0 {
                    return format!(
                        "{} {} ago",
                        mins,
                        if mins == 1 { "minute" } else { "minutes" }
                    );
                } else {
                    let seconds = offset.whole_seconds();
                    if seconds > 0 {
                        return format!(
                            "{} {} ago",
                            seconds,
                            if seconds == 1 { "second" } else { "seconds" }
                        );
                    }
                }
            }
        }
    }

    String::new()
}

fn generate_list_items(
    items: Vec<(usize, Post)>,
    current_time: time::OffsetDateTime,
) -> Vec<ListItem<'static>> {
    let list_items = {
        items
            .iter()
            .map(|(pos, post)| {
                let digits = pos.to_string().chars().count();
                let padding: String = vec![' '; digits].into_iter().collect();
                match post {
                    Post::Comment(comment) => PostItem::new(
                        Spans::from(vec![
                            Span::styled(format!("{}", pos), Style::default().fg(Color::Red)),
                            Span::raw(format!(" {:?}", comment)),
                        ]),
                        Spans::from(vec![Span::styled(
                            format!(
                                " {}by {} | {}",
                                padding,
                                comment.by,
                                get_time_offset(current_time, comment.time),
                            ),
                            Style::default().fg(Color::LightBlue),
                        )]),
                    ),
                    Post::Job(job) => {
                        let url = match extract_url(job.url.as_str()) {
                            Some(uri) => uri,
                            None => job.url.as_str(),
                        };

                        PostItem::new(
                            Spans::from(vec![
                                Span::styled(format!("{}", pos), Style::default().fg(Color::Red)),
                                Span::styled(format!(" {}", job.title.clone()), Style::default()),
                                Span::styled(
                                    format!(" ({})", url),
                                    Style::default()
                                        .fg(Color::Gray)
                                        .add_modifier(Modifier::ITALIC),
                                ),
                            ]),
                            Spans::from(vec![Span::styled(
                                format!(
                                    " {}by {} | {}",
                                    padding,
                                    job.by,
                                    get_time_offset(current_time, job.time),
                                ),
                                Style::default().fg(Color::LightBlue),
                            )]),
                        )
                    }
                    Post::Poll(poll) => {
                        let descendants = poll.descendants;
                        let points = poll.score;
                        PostItem::new(
                            Spans::from(vec![
                                Span::styled(format!("{}", pos), Style::default().fg(Color::Red)),
                                Span::raw(format!(" {}", poll.title.clone())),
                            ]),
                            Spans::from(vec![Span::styled(
                                format!(
                                    " {}{} {} by {} | {} | {} {}",
                                    padding,
                                    points,
                                    if points == 1 { "point" } else { "points" },
                                    poll.by,
                                    get_time_offset(current_time, poll.time),
                                    descendants,
                                    if descendants == 1 {
                                        "comment"
                                    } else {
                                        "comments"
                                    },
                                ),
                                Style::default().fg(Color::LightBlue),
                            )]),
                        )
                    }
                    Post::PollOpt(poll_opt) => {
                        let points = poll_opt.score;
                        PostItem::new(
                            Spans::from(vec![
                                Span::styled(format!("{}", pos), Style::default().fg(Color::Red)),
                                Span::raw(format!(" {:?}", poll_opt)),
                            ]),
                            Spans::from(vec![Span::styled(
                                format!(
                                    " {}{} {} by {} | {}",
                                    padding,
                                    points,
                                    if points == 1 { "point" } else { "points" },
                                    poll_opt.by,
                                    get_time_offset(current_time, poll_opt.time),
                                ),
                                Style::default().fg(Color::LightBlue),
                            )]),
                        )
                    }
                    Post::Story(story) => {
                        let url = match extract_url(story.url.as_str()) {
                            Some(uri) => uri,
                            None => story.url.as_str(),
                        };

                        let descendants = story.descendants;
                        let points = story.score;

                        PostItem::new(
                            Spans::from(vec![
                                Span::styled(format!("{}", pos), Style::default().fg(Color::Red)),
                                Span::styled(format!(" {}", story.title.clone()), Style::default()),
                                Span::styled(
                                    format!(" ({})", url),
                                    Style::default().add_modifier(Modifier::ITALIC),
                                ),
                            ]),
                            Spans::from(vec![Span::styled(
                                format!(
                                    " {}{} {} by {} | {} | {} {}",
                                    padding,
                                    points,
                                    if points == 1 { "point" } else { "points" },
                                    story.by,
                                    get_time_offset(current_time, story.time),
                                    descendants,
                                    if descendants == 1 {
                                        "comment"
                                    } else {
                                        "comments"
                                    },
                                ),
                                Style::default().fg(Color::LightBlue),
                            )]),
                        )
                    }
                }
            })
            .flat_map(Vec::<ListItem>::from)
            .collect::<Vec<ListItem>>()
    };

    list_items
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client: api::Client = api::Client::new(api::BASE_URL.to_string(), reqwest::Client::new());
    let mut top_items: TopItems = TopItems::new(client.clone());
    let mut new_items: NewItems = NewItems::new(client);
    let mut current_time: OffsetDateTime = OffsetDateTime::now_utc();
    let config = Config::new()?;

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    enable_raw_mode()?;

    let receiver = setup_input();

    let mut stateful_list = match config.default_view {
        MenuItem::Top => {
            top_items.list =
                generate_list_items(top_items.get_vec(config.max_items).await?, current_time);
            StatefulList::new(top_items.list.clone(), config.scroll_past_list)
        }
        MenuItem::New => {
            new_items.list =
                generate_list_items(new_items.get_vec(config.max_items).await?, current_time);
            StatefulList::new(new_items.list.clone(), config.scroll_past_list)
        }
    };
    stateful_list.next();

    let mut active_menu_item = config.default_view;
    let tabs: Vec<Spans> = ["Top", "New"].iter().cloned().map(Spans::from).collect();

    loop {
        terminal.draw(|frame| match active_menu_item {
            MenuItem::Top => {
                if stateful_list.items != top_items.list {
                    stateful_list.items = generate_list_items(
                        block_on(top_items.get_vec(config.max_items)).unwrap_or_else(|_| vec![]),
                        current_time,
                    );
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
                    stateful_list.items = generate_list_items(
                        block_on(new_items.get_vec(config.max_items)).unwrap_or_else(|_| vec![]),
                        current_time,
                    );
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
            Event::Input(event) => {
                if config.view_comments.contains(&event.code) {
                    if let Some(index) = stateful_list.state.selected() {
                        let items = match active_menu_item {
                            MenuItem::Top => {
                                let top_items = &mut top_items;
                                block_on(top_items.get_vec(config.max_items))?
                            }
                            MenuItem::New => {
                                let new_items = &mut new_items;
                                block_on(new_items.get_vec(config.max_items))?
                            }
                        };

                        if let Some(item) = items.get(index / 2) {
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
                } else if config.quit.contains(&event.code) {
                    disable_raw_mode()?;
                    terminal.clear()?;
                    terminal.show_cursor()?;
                    break;
                } else if config.open_article.contains(&event.code) {
                    if let Some(index) = stateful_list.state.selected() {
                        let items = match active_menu_item {
                            MenuItem::Top => {
                                let top_items = &mut top_items;
                                block_on(top_items.get_vec(config.max_items))?
                            }
                            MenuItem::New => {
                                let new_items = &mut new_items;
                                block_on(new_items.get_vec(config.max_items))?
                            }
                        };

                        if let Some(item) = items.get(index / 2) {
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
                } else if config.left.contains(&event.code) || config.right.contains(&event.code) {
                    active_menu_item.scroll();
                } else if config.up.contains(&event.code) {
                    stateful_list.previous();
                } else if config.down.contains(&event.code) {
                    stateful_list.next();
                } else if config.refresh.contains(&event.code) {
                    match active_menu_item {
                        MenuItem::Top => {
                            current_time = OffsetDateTime::now_utc();
                            top_items.refresh(config.max_items).await?;
                            top_items.list = generate_list_items(
                                top_items.get_vec(config.max_items).await?,
                                current_time,
                            );
                            stateful_list.items = top_items.list.clone();
                        }
                        MenuItem::New => {
                            current_time = OffsetDateTime::now_utc();
                            new_items.refresh(config.max_items).await?;
                            new_items.list = generate_list_items(
                                new_items.get_vec(config.max_items).await?,
                                current_time,
                            );
                            stateful_list.items = new_items.list.clone();
                        }
                    }
                }
            }
            Event::Tick => {}
        }
    }

    Ok(())
}
