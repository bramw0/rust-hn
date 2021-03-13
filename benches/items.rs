use api::Post;
use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;
use tui::{text::Span, widgets::ListItem};

fn get_items() {
    let client = api::Client::new();

    // 1.3753s
    // let a = client.get_item_by_id(26446070).unwrap();

    let top_stories: Vec<u32> = client
        .get_top_stories()
        .unwrap()
        .into_iter()
        .take(25)
        .collect();

    let items: Vec<ListItem> = {
        let mut vec = Vec::new();
        for id in top_stories {
            vec.push(client.get_item_by_id(id).unwrap())
        }

        vec.iter()
            .map(|post| match post {
                Post::Comment(comment) => ListItem::new(Span::raw(format!("{:?}", comment))),
                Post::Job(job) => ListItem::new(Span::raw(format!("{:?}", job))),
                Post::Poll(poll) => ListItem::new(Span::raw(format!("{:?}", poll))),
                Post::PollOpt(poll_opt) => ListItem::new(Span::raw(format!("{:?}", poll_opt))),
                Post::Story(story) => ListItem::new(Span::raw(format!("{:?}", story))),
            })
            .collect()
    };
}

async fn get_items_async() {
    let client = reqwest::Client::new();

    /* let item = reqwest::get("https://hacker-news.firebaseio.com/v0/item/26446070.json")
    .await
    .unwrap(); */

    let top_stories: Vec<u32> = client
        .get("https://hacker-news.firebaseio.com/v0/topstories.json")
        .send()
        .await
        .unwrap()
        .json::<Vec<u32>>()
        .await
        .unwrap()
        .into_iter()
        .take(25)
        .collect();

    let items: Vec<ListItem> = {
        let mut vec = Vec::new();
        for id in top_stories {
            vec.push(
                client
                    .get(&format!(
                        "https://hacker-news.firebaseio.com/v0/item/{}.json",
                        id
                    ))
                    .send()
                    .await
                    .unwrap()
                    .json::<Post>()
                    .await
                    .unwrap(),
            )
        }

        vec.iter()
            .map(|post| match post {
                Post::Comment(comment) => ListItem::new(Span::raw(format!("{:?}", comment))),
                Post::Job(job) => ListItem::new(Span::raw(format!("{:?}", job))),
                Post::Poll(poll) => ListItem::new(Span::raw(format!("{:?}", poll))),
                Post::PollOpt(poll_opt) => ListItem::new(Span::raw(format!("{:?}", poll_opt))),
                Post::Story(story) => ListItem::new(Span::raw(format!("{:?}", story))),
            })
            .collect()
    };
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Items");

    group.bench_function("items async", |b| {
        b.to_async(Runtime::new().unwrap())
            .iter(|| get_items_async())
    });
    group.bench_function("items", |b| b.iter(|| get_items()));

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(100);
    targets = criterion_benchmark
);
criterion_main!(benches);
