use std::{future::Future, io, pin::Pin, time::Duration};

use serde_json::json;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    time::timeout,
};

use crate::{Client, Difficulty, Error, Problem, ProblemList, Profile, client::http_builder};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type Progress = Vec<(usize, Option<u64>)>;
type RequestFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, Error>> + 'a>>;

async fn exchange<T>(
    response: String,
    operation: impl for<'a> FnOnce(&'a Client, &'a mut Progress) -> RequestFuture<'a, T>,
) -> TestResult<(Result<T, Error>, String, Progress)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let client = Client {
        http: http_builder()
            .https_only(false)
            .no_proxy()
            .timeout(Duration::from_secs(2))
            .build()?,
        endpoint: format!("http://{}/", listener.local_addr()?).into(),
    };
    let server = async {
        let (mut stream, _) = listener.accept().await?;
        let mut request = Vec::new();
        let mut buffer = [0; 2048];
        let end = loop {
            let read = stream.read(&mut buffer).await?;
            assert_ne!(read, 0, "client closed before sending its request");
            request.extend_from_slice(&buffer[..read]);
            assert!(request.len() < 16 * 1024);
            if let Some(end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                break end + 4;
            }
        };
        let headers = std::str::from_utf8(&request[..end - 4])?;
        let user_agent = format!("user-agent: {}/", env!("CARGO_PKG_NAME"));
        assert!(headers.starts_with("GET /rest/contests/master/"));
        assert!(
            headers
                .to_ascii_lowercase()
                .lines()
                .any(|line| line.starts_with(&user_agent))
        );
        assert!(!headers.to_ascii_lowercase().contains("\r\ncookie:"));
        if let Err(error) = stream.write_all(response.as_bytes()).await
            && !matches!(
                error.kind(),
                io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset
            )
        {
            return Err(error.into());
        }
        Ok::<String, Box<dyn std::error::Error>>(
            headers.lines().next().unwrap_or_default().to_owned(),
        )
    };
    let mut progress = Vec::new();
    let (result, request) = timeout(Duration::from_secs(3), async {
        tokio::join!(operation(&client, &mut progress), server)
    })
    .await?;
    Ok((result, request?, progress))
}

#[tokio::test]
async fn public_problem_contract() -> TestResult {
    let list = json!({"models": [{
        "id": 2532, "slug": "solve-me-first", "name": "Solve Me First",
        "difficulty_name": "Easy", "preview": "Add two integers",
        "track": {"slug": "warmup", "name": "Warmup", "track_slug": "algorithms", "track_name": "Algorithms"}
    }], "total": 10})
    .to_string();
    let show = json!({"status": true, "model": {
        "id": 2532, "slug": "solve-me-first", "name": "Solve Me First",
        "difficulty_name": "Easy", "preview": "Add two integers",
        "track": {"slug": "warmup", "name": "Warmup", "track_slug": "algorithms", "track_name": "Algorithms"},
        "problem_statement": "Compute $a + b$", "input_format": "Two integers", "output_format": "Their sum",
        "languages": ["cpp", "python3", "rust"],
        "onboarding": {
            "cpp": {"template": "int main() {}"},
            "statement": "Ignored metadata"
        },
"sample_test_cases": [{"input": "2\n7", "output": "9"}],
        "public_test_cases": true
    }})
    .to_string();
    let profile = json!({"model": {
        "id": 15795, "username": "hARRY", "name": "harry", "country": "India",
        "level": 4, "event_count": 7, "created_at": "2012-10-14T07:00:40.000Z"
    }})
    .to_string();

    for (kind, body) in [("list", list), ("show", show), ("profile", profile)] {
        let length = body.len();
        let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {length}\r\n\r\n{body}");
        let (result, request, progress) = exchange(response, move |client, progress| match kind {
            "list" => Box::pin(async move {
                let list: ProblemList = client
                    .list_track("algorithms", 0, 20, |bytes, total| {
                        progress.push((bytes, total))
                    })
                    .await?;
                assert_eq!(list.total, 10);
                assert_eq!(list.problems[0].slug.as_ref(), "solve-me-first");
                assert_eq!(list.problems[0].difficulty, Difficulty::Easy);
                Ok::<_, Error>(())
            }),
            _ => Box::pin(async move {
                if kind == "profile" {
                    let profile: Profile = client
                        .profile("hARRY", |bytes, total| progress.push((bytes, total)))
                        .await?;
                    assert_eq!(profile.username.as_ref(), "hARRY");
                    assert_eq!(profile.level, Some(4));
                    assert_eq!(profile.event_count, Some(7));
                    return Ok::<_, Error>(());
                }
                let problem: Problem = client
                    .problem("solve-me-first", |bytes, total| {
                        progress.push((bytes, total))
                    })
                    .await?;
                assert_eq!(problem.statement.as_ref(), "Compute $a + b$");
                assert_eq!(problem.starters.len(), 1);
                assert_eq!(problem.starters[0].language.as_ref(), "cpp");
                assert_eq!(problem.starters[0].template.as_ref(), "int main() {}");
                assert_eq!(problem.samples[0].input.as_ref(), "2\n7");
                assert_eq!(problem.samples[0].output.as_ref(), "9");
                assert!(problem.has_public_test_cases);
                assert_eq!(
                    problem
                        .languages
                        .iter()
                        .map(|language| language.as_ref())
                        .collect::<Vec<_>>(),
                    ["cpp", "python3", "rust"]
                );
                Ok::<_, Error>(())
            }),
        })
        .await?;
        result?;
        match kind {
            "list" => assert_eq!(
                request,
                "GET /rest/contests/master/tracks/algorithms/challenges?offset=0&limit=20&track_login=true HTTP/1.1"
            ),
            "show" => assert_eq!(
                request,
                "GET /rest/contests/master/challenges/solve-me-first HTTP/1.1"
            ),
            _ => assert_eq!(
                request,
                "GET /rest/contests/master/hackers/hARRY/profile HTTP/1.1"
            ),
        }
        assert_eq!(progress.first(), Some(&(0, Some(length as u64))));
        assert_eq!(progress.last(), Some(&(length, Some(length as u64))));
    }
    Ok(())
}
