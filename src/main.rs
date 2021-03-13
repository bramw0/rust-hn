mod ui;

use api::Post;
use crossterm::{
    event, event::Event as CEvent, terminal::disable_raw_mode, terminal::enable_raw_mode,
};
use psl::{List as PslList, Psl};
use std::str;
use std::{sync::mpsc, thread, time::Duration, time::Instant};

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols::DOT,
    terminal::Terminal,
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Tabs},
};

enum Event<I> {
    Input(I),
    Tick,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = api::Client::new();

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    enable_raw_mode()?;

    let receiver = setup_input();

    let top_stories: Vec<u32> = client.get_top_stories("orderBy=\"$key\"&limitToFirst=25")?;

    let items = {
        let mut vec = Vec::new();
        for id in top_stories {
            vec.push(client.get_item_by_id(id, "")?)
        }

        vec.iter()
            .map(|post| match post {
                Post::Comment(comment) => {
                    ListItem::new(Spans::from(vec![Span::raw(format!("{:?}", comment))]))
                }
                Post::Job(job) => {
                    let url: String = job.url.clone().split('/').take(1).collect::<String>();
                    ListItem::new(Spans::from(vec![
                        Span::styled(job.title.clone(), Style::default()),
                        Span::styled(
                            format!(
                                " ({})",
                                /* str::from_utf8(
                                    &PslList
                                        .domain(job.url.clone().as_bytes())
                                        .unwrap()
                                        .as_bytes()
                                )
                                .unwrap() */
                                url
                            ),
                            Style::default().fg(Color::Gray),
                        ),
                    ]))
                }
                Post::Poll(poll) => ListItem::new(Spans::from(vec![Span::raw(poll.title.clone())])),
                Post::PollOpt(poll_opt) => {
                    ListItem::new(Spans::from(vec![Span::raw(format!("{:?}", poll_opt))]))
                }
                Post::Story(story) => {
                    let url: String = story
                        .url
                        .clone()
                        .split("https://")
                        .collect::<String>()
                        .split('/')
                        .take(1)
                        .collect::<String>();
                    ListItem::new(Spans::from(vec![
                        Span::styled(story.title.clone(), Style::default()),
                        Span::styled(
                            format!(
                                " ({})",
                                /* str::from_utf8(
                                    &PslList
                                        .domain(story.url.clone().as_bytes())
                                        .unwrap()
                                        .as_bytes()
                                )
                                .unwrap() */
                                url
                            ),
                            Style::default().fg(Color::Gray),
                        ),
                    ]))
                }
            })
            .collect()
    };

    let mut stateful_list = ui::StatefulList::new(items);

    loop {
        terminal.draw(|frame| {
            let tabs: Vec<Spans> = ["Top", "New"].iter().cloned().map(Spans::from).collect();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
                .split(frame.size());

            let list = List::new(stateful_list.items.clone())
                .block(Block::default().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT))
                .highlight_style(Style::default().fg(Color::Green));

            let tab = Tabs::new(tabs)
                .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(DOT);

            frame.render_widget(tab, chunks[0]);
            frame.render_stateful_widget(list, chunks[1], &mut stateful_list.state);
        })?;

        match receiver.recv()? {
            Event::Input(event) => match event.code {
                event::KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                event::KeyCode::Char('j') => {
                    stateful_list.next();
                }
                event::KeyCode::Char('k') => {
                    stateful_list.previous();
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}
