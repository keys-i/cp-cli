use std::{future::Future, pin::Pin, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    time::timeout,
};

use crate::{Client, Discussion, DiscussionList, Error, UserStats, client::http_builder};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type RequestFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, Error>> + 'a>>;

async fn exchange<T>(
    response: String,
    operation: impl for<'a> FnOnce(&'a Client) -> RequestFuture<'a, T>,
) -> TestResult<(Result<T, Error>, String)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let client = Client {
        http: http_builder()
            .https_only(false)
            .no_proxy()
            .timeout(Duration::from_secs(2))
            .build()?,
        endpoint: format!("http://{}/", listener.local_addr()?).into(),
        profile_endpoint: format!("http://{}/", listener.local_addr()?).into(),
    };
    let server = async {
        let (mut stream, _) = listener.accept().await?;
        let mut request = Vec::new();
        let mut buffer = [0; 2048];
        loop {
            let read = stream.read(&mut buffer).await?;
            assert_ne!(read, 0, "client closed before sending its request");
            request.extend_from_slice(&buffer[..read]);
            if request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                break;
            }
        }
        assert!(
            request
                .windows(b"\r\ncookie:".len())
                .all(|window| !window.eq_ignore_ascii_case(b"\r\ncookie:"))
        );
        stream.write_all(response.as_bytes()).await?;
        Ok::<String, Box<dyn std::error::Error>>(
            std::str::from_utf8(&request)?
                .lines()
                .next()
                .unwrap_or_default()
                .to_owned(),
        )
    };
    let (result, request) = timeout(Duration::from_secs(3), async {
        tokio::join!(operation(&client), server)
    })
    .await?;
    Ok((result, request?))
}

#[tokio::test]
async fn public_discourse_contract() -> TestResult {
    let latest = r#"{"topic_list":{"topics":[{"id":42,"title":"A title","slug":"a-title","created_at":"2026-09-22T00:00:00.000Z","last_posted_at":"2026-09-22T01:00:00.000Z","views":12,"posts_count":2,"like_count":3,"posters":[{"user_id":7}]}]},"users":[{"id":7,"username":"chef"}]}"#;
    let topic = r#"{"id":42,"title":"A title","slug":"a-title","created_at":"2026-09-22T00:00:00.000Z","last_posted_at":"2026-09-22T01:00:00.000Z","views":12,"posts_count":2,"like_count":3,"post_stream":{"posts":[{"post_number":1,"cooked":"<p>Hello <strong>world</strong></p>","username":"chef"}]}}"#;
    for (kind, body) in [("latest", latest), ("topic", topic)] {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        );
        let (result, request) = exchange(response, move |client| match kind {
            "latest" => Box::pin(async move {
                let list: DiscussionList = client.latest(|_, _| {}).await?;
                assert_eq!(list.discussions.len(), 1);
                assert_eq!(list.discussions[0].author.as_deref(), Some("chef"));
                assert_eq!(list.discussions[0].posts, 2);
                Ok::<_, Error>(())
            }),
            _ => Box::pin(async move {
                let discussion: Discussion = client.discussion(42, |_, _| {}).await?;
                assert_eq!(discussion.summary.slug.as_ref(), "a-title");
                assert_eq!(
                    discussion.body_html.as_ref(),
                    "<p>Hello <strong>world</strong></p>"
                );
                Ok::<_, Error>(())
            }),
        })
        .await?;
        result?;
        assert_eq!(
            request,
            if kind == "latest" {
                "GET /latest.json HTTP/1.1"
            } else {
                "GET /t/42.json HTTP/1.1"
            }
        );
    }
    Ok(())
}

#[tokio::test]
async fn public_profile_contract() -> TestResult {
    let client = Client::new()?;
    assert_eq!(
        client.profile_url("chef_7")?.as_str(),
        "https://www.codechef.com/users/chef_7"
    );
    let body = r#"<h1 class="h2-style">Chef</h1><h3>Total Problems Solved: 12</h3><div class="rating-number"> 1800 </div><small>(Highest Rating 1900)</small>No. of Contests Participated: <b>7</b>"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let (result, request) = exchange(response, |client| {
        Box::pin(async move {
            let stats: UserStats = client.user_stats("chef_7", |_, _| {}).await?;
            assert_eq!(stats.name.as_ref(), "Chef");
            assert_eq!(stats.solved, 12);
            assert_eq!(stats.rating, Some(1800));
            assert_eq!(stats.highest_rating, Some(1900));
            assert_eq!(stats.contests, Some(7));
            Ok::<_, Error>(())
        })
    })
    .await?;
    result?;
    assert_eq!(request, "GET /users/chef_7 HTTP/1.1");
    Ok(())
}
