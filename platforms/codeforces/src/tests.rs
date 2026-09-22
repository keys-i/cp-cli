use std::{future::Future, io, pin::Pin, time::Duration};

use sha2::Digest;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    time::timeout,
};

use crate::{
    ApiCredentials, Client, Error,
    client::{MAX_RESPONSE_BYTES, http_builder},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type RequestFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, Error>> + 'a>>;

async fn exchange<T>(
    response: String,
    expected_path: &'static str,
    operation: impl for<'a> FnOnce(&'a Client) -> RequestFuture<'a, T>,
) -> TestResult<Result<T, Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let client = Client {
        http: http_builder()
            .https_only(false)
            .no_proxy()
            .timeout(Duration::from_secs(2))
            .build()?,
        endpoint: format!("http://{}/api", listener.local_addr()?).into(),
    };
    let server = async {
        let (mut stream, _) = listener.accept().await?;
        let mut request = Vec::new();
        let mut buffer = [0; 1024];
        loop {
            let read = stream.read(&mut buffer).await?;
            assert_ne!(read, 0, "client closed before sending its request");
            request.extend_from_slice(&buffer[..read]);
            if request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                break;
            }
        }
        assert!(request.starts_with(format!("GET {expected_path} HTTP/1.1\r\n").as_bytes()));
        stream.write_all(response.as_bytes()).await?;
        Ok::<(), io::Error>(())
    };
    let (result, _) = timeout(Duration::from_secs(3), async {
        tokio::join!(operation(&client), server)
    })
    .await?;
    Ok(result)
}

#[tokio::test]
async fn authorized_friends_contract() -> TestResult {
    let success = r#"{"status":"OK","result":["friend"]}"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{success}",
        success.len()
    );
    let credentials = ApiCredentials::new("abc123", "secret456")?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let client = Client {
        http: http_builder()
            .https_only(false)
            .no_proxy()
            .timeout(Duration::from_secs(2))
            .build()?,
        endpoint: format!("http://{}/api", listener.local_addr()?).into(),
    };
    let server = async {
        let (mut stream, _) = listener.accept().await?;
        let mut request = Vec::new();
        let mut buffer = [0; 1024];
        loop {
            let read = stream.read(&mut buffer).await?;
            assert_ne!(read, 0, "client closed before sending its request");
            request.extend_from_slice(&buffer[..read]);
            if request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                break;
            }
        }
        let request = std::str::from_utf8(&request)?;
        let target = request
            .split_whitespace()
            .next()
            .and_then(|method| (method == "GET").then(|| request.split_whitespace().nth(1)))
            .flatten()
            .ok_or("missing GET target")?;
        let query = target
            .strip_prefix("/api/user.friends?")
            .ok_or("wrong endpoint")?;
        let mut values = std::collections::BTreeMap::new();
        for parameter in query.split('&') {
            let (name, value) = parameter.split_once('=').ok_or("bad parameter")?;
            values.insert(name, value);
        }
        assert_eq!(values.get("apiKey"), Some(&"abc123"));
        assert_eq!(values.get("onlyOnline"), Some(&"true"));
        let time = values.get("time").ok_or("missing time")?;
        assert!(time.parse::<u64>()? > 0);
        let signature = values.get("apiSig").ok_or("missing signature")?;
        let (prefix, digest) = signature.split_at(6);
        assert!(prefix.bytes().all(|byte| byte.is_ascii_hexdigit()));
        let expected = sha2::Sha512::digest(
            format!("{prefix}/user.friends?apiKey=abc123&onlyOnline=true&time={time}#secret456")
                .as_bytes(),
        );
        assert_eq!(digest, format!("{expected:x}"));
        stream.write_all(response.as_bytes()).await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    };
    let (friends, _) = timeout(Duration::from_secs(3), async {
        tokio::join!(
            client.authenticated_friends(&credentials, |_, _| {}),
            server
        )
    })
    .await?;
    assert_eq!(friends?, vec!["friend".into()]);
    assert!(matches!(
        ApiCredentials::new("bad&key", "secret"),
        Err(Error::InvalidApiKey)
    ));
    Ok(())
}

#[tokio::test]
async fn problem_catalogue_contract() -> TestResult {
    let success = r#"{"status":"OK","result":{"problems":[{"contestId":1,"index":"A","name":"Theatre Square","rating":1000,"tags":["math"]}],"problemStatistics":[{"contestId":1,"index":"A","solvedCount":123}]}}"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{success}",
        success.len()
    );
    let problems = exchange(response, "/api/problemset.problems?lang=en", |client| {
        Box::pin(client.problems(|_, _| {}))
    })
    .await??;
    assert_eq!(problems.len(), 1);
    assert_eq!(
        problems[0].url.as_ref(),
        "https://codeforces.com/problemset/problem/1/A"
    );
    assert_eq!(problems[0].solved_count, Some(123));
    assert!(problems[0].statement.is_none());

    let failure = r#"{"status":"FAILED","comment":"Call limit exceeded"}"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{failure}",
        failure.len()
    );
    let error = exchange(response, "/api/problemset.problems?lang=en", |client| {
        Box::pin(client.problems(|_, _| {}))
    })
    .await?;
    assert!(matches!(error, Err(Error::ApiFailure)));

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n",
        MAX_RESPONSE_BYTES + 1
    );
    let error = exchange(response, "/api/problemset.problems?lang=en", |client| {
        Box::pin(client.problems(|_, _| {}))
    })
    .await?;
    assert!(matches!(error, Err(Error::ResponseTooLarge { .. })));

    let success = r#"{"status":"OK","result":[{"id":1,"name":"Round","type":"CF","phase":"BEFORE","startTimeSeconds":1,"durationSeconds":7200}]}"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{success}",
        success.len()
    );
    let contests = exchange(response, "/api/contest.list?gym=false", |client| {
        Box::pin(client.contests(|_, _| {}))
    })
    .await??;
    assert_eq!(contests[0].phase.as_ref(), "BEFORE");

    let success = r#"{"status":"OK","result":[{"handle":"tourist","contribution":1,"friendOfCount":2,"registrationTimeSeconds":3,"lastOnlineTimeSeconds":4}]}"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{success}",
        success.len()
    );
    let profile = exchange(response, "/api/user.info?handles=tourist", |client| {
        Box::pin(client.user_profile("tourist", |_, _| {}))
    })
    .await??;
    assert_eq!(profile.handle.as_ref(), "tourist");

    let success = r#"{"status":"OK","result":[{"contestId":1,"contestName":"Round","rank":2,"oldRating":1000,"newRating":1100,"ratingUpdateTimeSeconds":3}]}"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{success}",
        success.len()
    );
    let ratings = exchange(response, "/api/user.rating?handle=tourist", |client| {
        Box::pin(client.rating_history("tourist", |_, _| {}))
    })
    .await??;
    assert_eq!(ratings[0].new_rating, 1100);

    let success = r#"{"status":"OK","result":[{"id":1,"creationTimeSeconds":2,"relativeTimeSeconds":3,"problem":{"index":"A","name":"Problem"},"programmingLanguage":"Rust","testset":"TESTS","passedTestCount":4,"timeConsumedMillis":5,"memoryConsumedBytes":6}]}"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{success}",
        success.len()
    );
    let submissions = exchange(
        response,
        "/api/user.status?handle=tourist&from=1&count=1",
        |client| Box::pin(client.submissions("tourist", 1, |_, _| {})),
    )
    .await??;
    assert_eq!(submissions[0].problem_name.as_ref(), "Problem");

    let success = r#"{"status":"OK","result":[{"timeSeconds":10,"blogEntry":{"id":7,"creationTimeSeconds":8,"authorHandle":"tourist","title":"Notes","tags":["meta"],"rating":3},"comment":{"id":9,"creationTimeSeconds":10,"commentatorHandle":"reader","text":"Nice","rating":1}}]}"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{success}",
        success.len()
    );
    let discussions = exchange(response, "/api/recentActions?maxCount=20", |client| {
        Box::pin(client.recent_discussions(|_, _| {}))
    })
    .await??;
    assert_eq!(
        discussions[0]
            .latest_comment
            .as_ref()
            .map(|comment| comment.id),
        Some(9)
    );
    Ok(())
}
