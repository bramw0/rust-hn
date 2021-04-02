use api::Post;
use tui::widgets::ListItem;

pub struct TopItems {
    vec: Vec<(usize, Post)>,
    pub list: Vec<ListItem<'static>>,
    client: api::Client,
}

impl TopItems {
    pub fn new(client: api::Client) -> TopItems {
        TopItems {
            vec: Vec::new(),
            list: Vec::new(),
            client
        }
    }

    pub fn get_vec(&mut self) -> Result<Vec<(usize, Post)>, Box<dyn std::error::Error>> {
        if self.vec.is_empty() {
            self.vec = Self::get_items(self.client.clone())?;
        }

        Ok(self.vec.clone())
    }

    fn get_items(client: api::Client) -> Result<Vec<(usize, Post)>, Box<dyn std::error::Error>> {
        let stories = client.get_top_stories("orderBy=\"$key\"&limitToFirst=25")?;

        let items = construct_items(stories, client)?;

        Ok(items)
    }
}

pub struct NewItems {
    vec: Vec<(usize, Post)>,
    pub list: Vec<ListItem<'static>>,
    client: api::Client,
}

impl NewItems {
    pub fn new(client: api::Client) -> NewItems {
        NewItems {
            vec: Vec::new(),
            list: Vec::new(),
            client
        }
    }

    pub fn get_vec(&mut self) -> Result<Vec<(usize, Post)>, Box<dyn std::error::Error>> {
        if self.vec.is_empty() {
            self.vec = Self::get_items(self.client.clone())?;
        }

        Ok(self.vec.clone())
    }


    fn get_items(client: api::Client) -> Result<Vec<(usize, Post)>, Box<dyn std::error::Error>> {
        let stories = client.get_new_stories("orderBy=\"$key\"&limitToFirst=25")?;

        let items = construct_items(stories, client)?;

        Ok(items)
    }
}

fn construct_items(stories: Vec<u32>, client: api::Client) -> Result<Vec<(usize, Post)>, Box<dyn std::error::Error>> {
    let items: Vec<(usize, Post)> = {
        let mut vec = Vec::new();
        for (pos, id) in stories.iter().enumerate() {
            let it = std::time::Instant::now();
            vec.push((pos + 1, client.get_item_by_id(*id, "")?));
            eprintln!("{}:\t{:?}", pos, it.elapsed())
        }

        vec
    };

    Ok(items)
}