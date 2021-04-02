use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum Post {
    Job(Job),
    Story(Story),
    Comment(Comment),
    Poll(Poll),
    PollOpt(PollOpt),
}

#[derive(Debug, Deserialize, Clone)]
pub struct PollOpt {
    pub id: u32,
    #[serde(default)]
    pub deleted: bool,
    #[serde(default)]
    pub by: String,
    #[serde(with = "time::serde::timestamp")]
    #[serde(default = "time::OffsetDateTime::now_utc")]
    pub time: time::OffsetDateTime,
    #[serde(default)]
    pub dead: bool,
    #[serde(default)]
    pub kids: Vec<u32>,
    #[serde(default)]
    pub parent: u32,
    #[serde(default)]
    pub score: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Poll {
    pub id: u32,
    #[serde(default)]
    pub deleted: bool,
    #[serde(default)]
    pub by: String,
    #[serde(with = "time::serde::timestamp")]
    #[serde(default = "time::OffsetDateTime::now_utc")]
    pub time: time::OffsetDateTime,
    #[serde(default)]
    pub dead: bool,
    #[serde(default)]
    pub kids: Vec<u32>,
    #[serde(default)]
    pub parts: Vec<u32>,
    #[serde(default)]
    pub descendants: u32,
    #[serde(default)]
    pub score: u32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Comment {
    pub id: u32,
    #[serde(default)]
    pub deleted: bool,
    #[serde(default)]
    pub by: String,
    #[serde(with = "time::serde::timestamp")]
    #[serde(default = "time::OffsetDateTime::now_utc")]
    pub time: time::OffsetDateTime,
    #[serde(default)]
    pub dead: bool,
    #[serde(default)]
    pub kids: Vec<u32>,
    #[serde(default)]
    pub parent: u32,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Job {
    pub id: u32,
    #[serde(default)]
    pub deleted: bool,
    #[serde(default)]
    pub by: String,
    #[serde(with = "time::serde::timestamp")]
    #[serde(default = "time::OffsetDateTime::now_utc")]
    pub time: time::OffsetDateTime,
    #[serde(default)]
    pub dead: bool,
    #[serde(default)]
    pub kids: Vec<u32>,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Story {
    pub id: u32,
    #[serde(default)]
    pub deleted: bool,
    #[serde(default)]
    pub by: String,
    #[serde(with = "time::serde::timestamp")]
    #[serde(default = "time::OffsetDateTime::now_utc")]
    pub time: time::OffsetDateTime,
    #[serde(default)]
    pub dead: bool,
    #[serde(default)]
    pub kids: Vec<u32>,
    #[serde(default)]
    pub descendants: u32,
    #[serde(default)]
    pub score: u32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub id: String,
    #[serde(with = "time::serde::timestamp")]
    #[serde(default = "time::OffsetDateTime::now_utc")]
    pub created: time::OffsetDateTime,
    pub karma: u32,
    #[serde(default)]
    pub about: String,
    #[serde(default)]
    pub submitted: Vec<u32>,
}

impl Default for User {
    fn default() -> Self {
        User {
            id: String::default(),
            created: time::OffsetDateTime::now_utc(),
            karma: u32::default(),
            about: String::default(),
            submitted: Vec::default(),
        }
    }
}

#[derive(Clone)]
pub struct Client {
    pub agent: ureq::Agent,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Updates {
    #[serde(default)]
    pub items: Vec<u32>,
    #[serde(default)]
    pub profiles: Vec<String>,
}

/* lazy_static! {
    static ref CLIENT: Client = Client::new();
} */

#[allow(dead_code)]
pub const BASE_URL: &str = "https://hacker-news.firebaseio.com/v0";

impl Default for Client {
    fn default() -> Self {
        Self::new(BASE_URL.to_string(), ureq::Agent::new())
    }
}

impl Client {
    pub fn new(url: String, agent: ureq::Agent) -> Self {
        Self { agent, url }
    }

    pub fn perform_request(&self, url: &str) -> Result<ureq::Response, Box<dyn std::error::Error>> {
        Ok(self.agent.get(url).call()?)
    }

    pub fn get_item_by_id(
        &self,
        id: u32,
        options: &str,
    ) -> Result<Post, Box<dyn std::error::Error>> {
        Ok(self
            .perform_request(&format!("{}/item/{}.json?{}", self.url, id, options))?
            .into_json::<Post>()?)
    }

    pub fn get_user_by_id(
        &self,
        id: &str,
        options: &str,
    ) -> Result<User, Box<dyn std::error::Error>> {
        Ok(self
            .perform_request(&format!("{}/user/{}.json?{}", self.url, id, options))?
            .into_json::<User>()?)
    }

    pub fn get_max_item_id(&self, options: &str) -> Result<u32, Box<dyn std::error::Error>> {
        Ok(self
            .perform_request(&format!("{}/maxitem.json?{}", self.url, options))?
            .into_json::<u32>()?)
    }

    pub fn get_top_stories(&self, options: &str) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        Ok(self
            .perform_request(&format!("{}/topstories.json?{}", self.url, options))?
            .into_json::<Vec<u32>>()?)
    }

    pub fn get_new_stories(&self, options: &str) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        Ok(self
            .perform_request(&format!("{}/newstories.json?{}", self.url, options))?
            .into_json::<Vec<u32>>()?)
    }

    pub fn get_ask_stories(&self, options: &str) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        Ok(self
            .perform_request(&format!("{}/askstories.json?{}", self.url, options))?
            .into_json::<Vec<u32>>()?)
    }

    pub fn get_show_stories(&self, options: &str) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        Ok(self
            .perform_request(&format!("{}/showstories.json?{}", self.url, options))?
            .into_json::<Vec<u32>>()?)
    }

    pub fn get_job_stories(&self, options: &str) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        Ok(self
            .perform_request(&format!("{}/jobstories.json?{}", self.url, options))?
            .into_json::<Vec<u32>>()?)
    }

    pub fn get_updates(&self, options: &str) -> Result<Updates, Box<dyn std::error::Error>> {
        Ok(self
            .perform_request(&format!("{}/updates.json?{}", self.url, options))?
            .into_json::<Updates>()?)
    }
}

/* pub fn from_id<'de, D>(deserializer: D) -> Result<User, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let id: String = serde::Deserialize::deserialize(deserializer).unwrap();

    Ok(CLIENT.get_user_by_id(id.as_str()).unwrap())
} */
