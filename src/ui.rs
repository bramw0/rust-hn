use tui::{
    text::Text,
    widgets::{ListItem, ListState},
};

#[derive(Clone, Debug, PartialEq)]
pub struct PostItem<'a> {
    pub top_item: ListItem<'a>,
    pub bottom_item: ListItem<'a>,
}

impl<'a> From<PostItem<'a>> for Vec<ListItem<'a>> {
    fn from(post_item: PostItem<'a>) -> Self {
        vec![post_item.top_item, post_item.bottom_item]
    }
}

impl<'a> PostItem<'a> {
    pub fn new<T>(content_top: T, content_bottom: T) -> PostItem<'a>
    where
        T: Into<Text<'a>>,
    {
        PostItem {
            top_item: ListItem::new(content_top),
            bottom_item: ListItem::new(content_bottom),
        }
    }
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
    scroll_past_list: bool,
}

impl<T> StatefulList<T>
where
    T: Clone,
{
    pub fn new(items: Vec<T>, scroll_past_list: bool) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
            scroll_past_list,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if self.scroll_past_list {
                    if i >= self.items.len() - 1 {
                        0
                    } else {
                        i + 2
                    }
                } else {
                    if i >= self.items.len() - 2 {
                        0
                    } else {
                        i + 2
                    }
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    if self.scroll_past_list {
                        self.items.len()
                    } else {
                        self.items.len() - 2
                    }
                } else {
                    i - 2
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
    Top,
    New,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Top => 0,
            MenuItem::New => 1,
        }
    }
}

impl MenuItem {
    pub fn scroll(&mut self) {
        match *self {
            MenuItem::Top => {
                *self = MenuItem::New;
            }
            MenuItem::New => {
                *self = MenuItem::Top;
            }
        }
    }
}
