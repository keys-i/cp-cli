use std::{future::Future, io, pin::Pin, time::Duration};

use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    time::timeout,
};

use crate::{
    Client, Difficulty, Error, Problem,
    client::{MAX_RESPONSE_BYTES, http_builder},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type Progress = Vec<(usize, Option<u64>)>;

#[derive(Clone, Copy, Debug)]
enum Failure {
    Decode,
    Graphql,
    InvalidResponse,
    NotFound,
    PremiumRequired,
}

fn problem_body() -> Value {
    json!({"data": {"question": {
        "titleSlug": "two-sum", "title": "Two Sum",
        "content": "<p>Find two numbers.</p>", "isPaidOnly": false
    }}})
}

fn has_failure(result: &Result<Problem, Error>, expected: Failure) -> bool {
    matches!(
        (result, expected),
        (Err(Error::Decode(_)), Failure::Decode)
            | (Err(Error::Graphql), Failure::Graphql)
            | (Err(Error::InvalidResponse), Failure::InvalidResponse)
            | (Err(Error::NotFound), Failure::NotFound)
            | (Err(Error::PremiumRequired), Failure::PremiumRequired)
    )
}

type RequestFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, Error>> + 'a>>;

async fn exchange<T>(
    response: String,
    operation: impl for<'a> FnOnce(&'a Client, &'a mut Progress) -> RequestFuture<'a, T>,
) -> TestResult<(Result<T, Error>, Value, Progress)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let client = Client {
        http: http_builder()
            .https_only(false)
            .no_proxy()
            .timeout(Duration::from_secs(2))
            .build()?,
        endpoint: format!("http://{}/graphql/", listener.local_addr()?).into(),
    };
    let server = async {
        let (mut stream, _) = listener.accept().await?;
        let mut request = Vec::new();
        let mut buffer = [0; 4096];
        let (header_end, length) = loop {
            let read = stream.read(&mut buffer).await?;
            assert_ne!(read, 0, "client closed before sending its request");
            request.extend_from_slice(&buffer[..read]);
            assert!(request.len() < 16 * 1024);
            if let Some(end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                let headers = std::str::from_utf8(&request[..end])?;
                assert!(headers.starts_with("POST /graphql/ HTTP/1.1\r\n"));
                let headers = headers.to_ascii_lowercase();
                for expected in [
                    "content-type: application/json",
                    "origin: https://leetcode.com",
                    "referer: https://leetcode.com/",
                    "user-agent: platform-leetcode/",
                ] {
                    assert!(headers.lines().any(|line| line.starts_with(expected)));
                }
                assert!(!headers.contains("\r\ncookie:"));
                assert!(!headers.contains("\r\nauthorization:"));
                let length = headers
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length: "))
                    .ok_or("missing request content length")?
                    .parse::<usize>()?;
                if request.len() >= end + 4 + length {
                    break (end + 4, length);
                }
            }
        };
        let request = serde_json::from_slice(&request[header_end..header_end + length])?;
        if let Err(error) = stream.write_all(response.as_bytes()).await
            && !matches!(
                error.kind(),
                io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset
            )
        {
            return Err(error.into());
        }
        Ok::<Value, Box<dyn std::error::Error>>(request)
    };
    let mut progress = Vec::new();
    let (result, request) = timeout(Duration::from_secs(3), async {
        tokio::join!(operation(&client, &mut progress), server)
    })
    .await?;
    Ok((result, request?, progress))
}

async fn query(body: Value) -> TestResult<Result<Problem, Error>> {
    let body = body.to_string();
    Ok(exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        ),
        |client, _| Box::pin(client.problem("two-sum", |_, _| {})),
    )
    .await?
    .0)
}

#[tokio::test]
async fn client_contract() -> TestResult {
    let body = problem_body().to_string();
    let length = body.len();
    let (result, request, progress) = exchange(
        format!("HTTP/1.1 200 OK\r\nContent-Length: {length}\r\n\r\n{body}"),
        |client, progress| {
            Box::pin(client.problem("two-sum", |bytes, total| progress.push((bytes, total))))
        },
    )
    .await?;
    let problem = result?;
    assert_eq!(problem.id.as_ref(), "two-sum");
    assert_eq!(problem.title.as_ref(), "Two Sum");
    assert_eq!(problem.statement.as_ref(), "<p>Find two numbers.</p>");
    assert_eq!(request["operationName"], "questionData");
    assert_eq!(request["variables"], json!({"titleSlug": "two-sum"}));
    assert!(request["query"].as_str().is_some_and(|query| {
        query.contains("question(titleSlug: $titleSlug)")
            && query.contains("titleSlug title content isPaidOnly")
    }));
    assert_eq!(progress.first(), Some(&(0, Some(length as u64))));
    assert_eq!(progress.last(), Some(&(length, Some(length as u64))));
    assert!(progress.windows(2).all(|pair| pair[0].0 <= pair[1].0));

    let search_body = json!({"data": {"problemsetQuestionList": {
        "total": 2,
        "questions": [
            {"frontendQuestionId": "1", "title": "Two Sum", "titleSlug": "two-sum", "difficulty": "Easy", "isPaidOnly": false},
            {"frontendQuestionId": "2", "title": "Add Two Numbers", "titleSlug": "add-two-numbers", "difficulty": "Medium", "isPaidOnly": true}
        ]
    }}})
    .to_string();
    let search_length = search_body.len();
    let (result, request, progress) = exchange(
        format!("HTTP/1.1 200 OK\r\nContent-Length: {search_length}\r\n\r\n{search_body}"),
        |client, progress| {
            Box::pin(client.search("  two sum  ", |bytes, total| progress.push((bytes, total))))
        },
    )
    .await?;
    let search = result?;
    assert_eq!(search.total, 2);
    assert_eq!(
        search.problems,
        vec![
            crate::ProblemSummary {
                number: 1,
                id: "two-sum".into(),
                title: "Two Sum".into(),
                difficulty: Difficulty::Easy,
                paid_only: false,
            },
            crate::ProblemSummary {
                number: 2,
                id: "add-two-numbers".into(),
                title: "Add Two Numbers".into(),
                difficulty: Difficulty::Medium,
                paid_only: true,
            },
        ]
    );
    assert_eq!(request["operationName"], "problemsetQuestionList");
    assert_eq!(
        request["variables"],
        json!({"categorySlug": "", "skip": 0, "limit": 20, "filters": {"searchKeywords": "two sum"}})
    );
    assert!(request["query"].as_str().is_some_and(|query| {
        query.contains("problemsetQuestionList: questionList")
            && query.contains("total: totalNum")
            && query.contains("questions: data")
            && query.contains("frontendQuestionId: questionFrontendId")
    }));
    assert_eq!(progress.first(), Some(&(0, Some(search_length as u64))));
    assert_eq!(
        progress.last(),
        Some(&(search_length, Some(search_length as u64)))
    );

    let mut graphql = problem_body();
    graphql["errors"] = json!([{"message": "sensitive server detail"}]);
    let mut missing_content = problem_body();
    missing_content["data"]["question"]["content"] = Value::Null;
    let mut premium = missing_content.clone();
    premium["data"]["question"]["isPaidOnly"] = json!(true);
    for (name, body, expected) in [
        (
            "not found",
            json!({"data": {"question": null}}),
            Failure::NotFound,
        ),
        ("null data", json!({"data": null}), Failure::InvalidResponse),
        ("missing question", json!({"data": {}}), Failure::Decode),
        ("graphql", graphql, Failure::Graphql),
        ("missing content", missing_content, Failure::InvalidResponse),
        ("premium", premium, Failure::PremiumRequired),
    ] {
        let result = query(body).await?;
        assert!(has_failure(&result, expected), "case={name}: {result:?}");
        if matches!(expected, Failure::Graphql) {
            assert!(
                !result
                    .err()
                    .ok_or("missing error")?
                    .to_string()
                    .contains("sensitive")
            );
        }
    }
    for (field, value) in [
        ("titleSlug", "another-problem"),
        ("title", ""),
        ("title", " "),
        ("content", ""),
        ("content", " "),
    ] {
        let mut body = problem_body();
        body["data"]["question"][field] = json!(value);
        let result = query(body).await?;
        assert!(
            has_failure(&result, Failure::InvalidResponse),
            "field={field}"
        );
    }

    let client = Client::new()?;
    for slug in ["", "../two-sum", "two sum", "two/sum", "two\nsum", "é"] {
        assert!(matches!(
            client.problem(slug, |_, _| {}).await,
            Err(Error::InvalidSlug)
        ));
    }
    for length in [129, 256, 1024] {
        assert!(matches!(
            client.problem(&"a".repeat(length), |_, _| {}).await,
            Err(Error::InvalidSlug)
        ));
    }
    for query in ["", " \t ", "two\nsum", "café", &"a".repeat(101)] {
        assert!(matches!(
            client.search(query, |_, _| {}).await,
            Err(Error::InvalidQuery)
        ));
    }

    for (field, value) in [
        ("frontendQuestionId", json!("0")),
        ("frontendQuestionId", json!("one")),
        ("title", json!(" ")),
        ("title", json!("Two\nSum")),
        ("title", json!(" Two Sum")),
        ("titleSlug", json!("two sum")),
        ("difficulty", json!("Unknown")),
    ] {
        let body = json!({"data": {"problemsetQuestionList": {
            "total": 1,
            "questions": [{
                "frontendQuestionId": "1", "title": "Two Sum", "titleSlug": "two-sum",
                "difficulty": "Hard", "isPaidOnly": false
            }]
        }}});
        let mut body = body;
        body["data"]["problemsetQuestionList"]["questions"][0][field] = value;
        let body = body.to_string();
        let (result, _, _) = exchange(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            ),
            |client, _| Box::pin(client.search("two", |_, _| {})),
        )
        .await?;
        assert!(
            matches!(result, Err(Error::InvalidResponse)),
            "field={field}"
        );
    }
    let body = json!({"data": {"problemsetQuestionList": {"total": 0, "questions": [{
        "frontendQuestionId": "1", "title": "Two Sum", "titleSlug": "two-sum",
        "difficulty": "Easy", "isPaidOnly": false
    }]}}})
    .to_string();
    let (result, _, _) = exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        ),
        |client, _| Box::pin(client.search("two", |_, _| {})),
    )
    .await?;
    assert!(matches!(result, Err(Error::InvalidResponse)));

    for status in [
        300, 301, 302, 307, 308, 400, 401, 403, 404, 408, 429, 500, 503,
    ] {
        let (result, _, progress) = exchange(
            format!(
                "HTTP/1.1 {status} Test\r\nLocation: http://127.0.0.1:1/\r\nContent-Length: 0\r\n\r\n"
            ),
            |client, _| Box::pin(client.problem("two-sum", |_, _| {})),
        )
        .await?;
        assert!(matches!(result, Err(Error::Status(code)) if code.as_u16() == status));
        assert!(progress.is_empty());
    }
    let (result, _, progress) = exchange(
        "HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\nnope".into(),
        |client, progress| {
            Box::pin(client.problem("two-sum", |bytes, total| progress.push((bytes, total))))
        },
    )
    .await?;
    assert!(matches!(result, Err(Error::Decode(_))));
    assert_eq!(progress, [(0, Some(4)), (4, Some(4))]);

    let (result, _, progress) = exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n",
            MAX_RESPONSE_BYTES + 1
        ),
        |client, progress| {
            Box::pin(client.problem("two-sum", |bytes, total| progress.push((bytes, total))))
        },
    )
    .await?;
    assert!(matches!(result, Err(Error::ResponseTooLarge { .. })));
    assert!(progress.is_empty());

    let mut body = problem_body().to_string();
    body.push_str(&" ".repeat(MAX_RESPONSE_BYTES - body.len()));
    let (result, _, progress) = exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        ),
        |client, progress| {
            Box::pin(client.problem("two-sum", |bytes, total| progress.push((bytes, total))))
        },
    )
    .await?;
    assert!(result.is_ok());
    assert_eq!(
        progress.last(),
        Some(&(MAX_RESPONSE_BYTES, Some(MAX_RESPONSE_BYTES as u64)))
    );
    let (result, _, progress) = exchange(
        format!(
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{body}\r\n1\r\n \r\n0\r\n\r\n",
            body.len()
        ),
        |client, progress| {
            Box::pin(client.problem("two-sum", |bytes, total| progress.push((bytes, total))))
        },
    )
    .await?;
    assert!(matches!(result, Err(Error::ResponseTooLarge { .. })));
    assert_eq!(progress.first(), Some(&(0, None)));
    assert!(progress.iter().all(|(_, total)| total.is_none()));

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let mut client = Client::new()?;
    client.endpoint = format!("http://{}/graphql/", listener.local_addr()?).into();
    assert!(
        matches!(client.problem("two-sum", |_, _| {}).await, Err(Error::Transport(error)) if error.is_builder())
    );
    client.http = http_builder()
        .https_only(false)
        .no_proxy()
        .timeout(Duration::from_millis(50))
        .build()?;
    let server = async {
        let (_stream, _) = listener.accept().await?;
        tokio::time::sleep(Duration::from_millis(150)).await;
        Ok::<_, io::Error>(())
    };
    let (result, server) = timeout(Duration::from_secs(3), async {
        tokio::join!(client.problem("two-sum", |_, _| {}), server)
    })
    .await?;
    server?;
    assert!(matches!(result, Err(Error::Transport(error)) if error.is_timeout()));
    Ok(())
}
