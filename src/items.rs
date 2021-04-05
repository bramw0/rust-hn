use api::Post;
use futures::{stream, StreamExt};
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
            client,
        }
    }

    pub async fn get_vec(
        &mut self,
        item_length: u8,
    ) -> Result<Vec<(usize, Post)>, Box<dyn std::error::Error>> {
        if self.vec.is_empty() {
            self.vec = Self::get_items(self.client.clone(), item_length).await?;
        }

        Ok(self.vec.clone())
    }

    async fn get_items(
        client: api::Client,
        item_length: u8,
    ) -> Result<Vec<(usize, Post)>, Box<dyn std::error::Error>> {
        let stories = client
            .get_top_stories(&format!("orderBy=\"$key\"&limitToFirst={}", item_length))
            .await?;

        let items = construct_items(stories, client).await?;

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
            client,
        }
    }

    pub async fn get_vec(&mut self) -> Result<Vec<(usize, Post)>, Box<dyn std::error::Error>> {
        if self.vec.is_empty() {
            self.vec = Self::get_items(self.client.clone()).await?;
        }

        Ok(self.vec.clone())
    }

    async fn get_items(
        client: api::Client,
    ) -> Result<Vec<(usize, Post)>, Box<dyn std::error::Error>> {
        let stories = client
            .get_new_stories("orderBy=\"$key\"&limitToFirst=25")
            .await?;

        let items = construct_items(stories, client).await?;

        Ok(items)
    }
}

async fn construct_items(
    stories: Vec<u32>,
    client: api::Client,
) -> Result<Vec<(usize, Post)>, Box<dyn std::error::Error>> {
    let mut items = Vec::new();

    let requests = stream::iter(stories.clone())
        .enumerate()
        .map(|(pos, id)| {
            let client = &client;
            async move {
                let post = client.get_item_by_id(id, "").await.unwrap();
                (pos + 1, post)
            }
        })
        .buffer_unordered(stories.len());

    #[cfg(debug_assertions)]
    let a = std::time::Instant::now();

    requests
        .fold(&mut items, |items, (pos, post)| async move {
            items.push((pos, post));
            items
        })
        .await;

    #[cfg(debug_assertions)]
    eprintln!("Total time: {:?}", a.elapsed());

    items.sort_by_key(|key| key.0);

    Ok(items)
}
