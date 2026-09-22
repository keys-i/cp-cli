use std::{future::Future, io, pin::Pin, time::Duration};

use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    time::timeout,
};

use crate::{
    Client, Credentials, Difficulty, Error, Problem, RunState, SubmissionState,
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
        "frontendQuestionId": "1", "titleSlug": "two-sum", "title": "Two Sum",
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
                let user_agent = format!("user-agent: {}/", env!("CARGO_PKG_NAME"));
                for expected in [
                    "content-type: application/json",
                    "origin: https://leetcode.com",
                    "referer: https://leetcode.com/",
                ] {
                    assert!(headers.lines().any(|line| line.starts_with(expected)));
                }
                assert!(headers.lines().any(|line| line.starts_with(&user_agent)));
                assert!(!headers.contains("\r\ncookie:"));
                assert!(!headers.contains("\r\nx-csrftoken:"));
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

async fn authenticated_exchange<T>(
    response: String,
    method: &'static str,
    path: &'static str,
    referer: &'static str,
    operation: impl for<'a> FnOnce(&'a Client, &'a Credentials) -> RequestFuture<'a, T>,
) -> TestResult<(Result<T, Error>, Value)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let client = Client {
        http: http_builder()
            .https_only(false)
            .no_proxy()
            .timeout(Duration::from_secs(2))
            .build()?,
        endpoint: format!("http://{}/graphql/", listener.local_addr()?).into(),
    };
    let credentials = Credentials::new("session-token", "csrf-token")?;
    let server = async {
        let (mut stream, _) = listener.accept().await?;
        let mut request = Vec::new();
        let mut buffer = [0; 4096];
        let header_end = loop {
            let read = stream.read(&mut buffer).await?;
            assert_ne!(read, 0, "client closed before sending its request");
            request.extend_from_slice(&buffer[..read]);
            assert!(request.len() < 16 * 1024);
            if let Some(end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                break end + 4;
            }
        };
        let headers = std::str::from_utf8(&request[..header_end - 4])?.to_ascii_lowercase();
        assert!(headers.starts_with(&format!("{method} {path} http/1.1\r\n").to_ascii_lowercase()));
        for expected in [
            "cookie: leetcode_session=session-token; csrftoken=csrf-token",
            "x-csrftoken: csrf-token",
            "origin: https://leetcode.com",
        ] {
            assert!(headers.lines().any(|line| line.starts_with(expected)));
        }
        assert!(
            headers
                .lines()
                .any(|line| line == format!("referer: {referer}").to_ascii_lowercase())
        );
        assert!(!headers.contains("\r\nauthorization:"));
        let length = headers
            .lines()
            .find_map(|line| line.strip_prefix("content-length: "))
            .map(str::parse::<usize>)
            .transpose()?
            .unwrap_or(0);
        while request.len() < header_end + length {
            let read = stream.read(&mut buffer).await?;
            assert_ne!(read, 0, "client closed before sending its request body");
            request.extend_from_slice(&buffer[..read]);
        }
        let body = if length == 0 {
            Value::Null
        } else {
            serde_json::from_slice(&request[header_end..header_end + length])?
        };
        stream.write_all(response.as_bytes()).await?;
        Ok::<Value, Box<dyn std::error::Error>>(body)
    };
    let (result, request) = timeout(Duration::from_secs(3), async {
        tokio::join!(operation(&client, &credentials), server)
    })
    .await?;
    Ok((result, request?))
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
    assert_eq!(problem.number, 1);
    assert_eq!(problem.id.as_ref(), "two-sum");
    assert_eq!(problem.title.as_ref(), "Two Sum");
    assert_eq!(problem.statement.as_ref(), "<p>Find two numbers.</p>");
    assert_eq!(request["operationName"], "questionData");
    assert_eq!(request["variables"], json!({"titleSlug": "two-sum"}));
    assert!(request["query"].as_str().is_some_and(|query| {
        query.contains("question(titleSlug: $titleSlug)")
            && query.contains(
                "frontendQuestionId: questionFrontendId titleSlug title content isPaidOnly",
            )
    }));
    assert_eq!(progress.first(), Some(&(0, Some(length as u64))));
    assert_eq!(progress.last(), Some(&(length, Some(length as u64))));
    assert!(progress.windows(2).all(|pair| pair[0].0 <= pair[1].0));

    let daily_body = json!({"data": {"activeDailyCodingChallengeQuestion": {"question": {
        "frontendQuestionId": "1", "titleSlug": "two-sum", "title": "Two Sum",
        "content": "<p>Find two numbers.</p>", "isPaidOnly": false
    }}}})
    .to_string();
    let daily_length = daily_body.len();
    let (result, request, progress) = exchange(
        format!("HTTP/1.1 200 OK\r\nContent-Length: {daily_length}\r\n\r\n{daily_body}"),
        |client, progress| Box::pin(client.daily(|bytes, total| progress.push((bytes, total)))),
    )
    .await?;
    let daily = result?;
    assert_eq!(daily.number, 1);
    assert_eq!(daily.id.as_ref(), "two-sum");
    assert_eq!(daily.title.as_ref(), "Two Sum");
    assert_eq!(daily.statement.as_ref(), "<p>Find two numbers.</p>");
    assert_eq!(request["operationName"], "questionOfToday");
    assert!(request.get("variables").is_none());
    assert!(request["query"].as_str().is_some_and(|query| {
        query.contains("activeDailyCodingChallengeQuestion")
            && query.contains(
                "question { frontendQuestionId: questionFrontendId titleSlug title content isPaidOnly }"
            )
    }));
    assert_eq!(progress.first(), Some(&(0, Some(daily_length as u64))));
    assert_eq!(
        progress.last(),
        Some(&(daily_length, Some(daily_length as u64)))
    );

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

    let (result, request, _) = exchange(
        format!("HTTP/1.1 200 OK\r\nContent-Length: {search_length}\r\n\r\n{search_body}"),
        |client, _| Box::pin(client.list(None, None, |_, _| {})),
    )
    .await?;
    assert_eq!(result?.total, 2);
    assert_eq!(request["operationName"], "problemsetQuestionList");
    assert_eq!(
        request["variables"],
        json!({"categorySlug": "", "skip": 0, "limit": 20, "filters": {}})
    );

    let (result, request, _) = exchange(
        format!("HTTP/1.1 200 OK\r\nContent-Length: {search_length}\r\n\r\n{search_body}"),
        |client, _| Box::pin(client.list(Some(Difficulty::Hard), Some("graph"), |_, _| {})),
    )
    .await?;
    assert_eq!(result?.total, 2);
    assert_eq!(
        request["variables"],
        json!({"categorySlug": "", "skip": 0, "limit": 20, "filters": {"difficulty": "HARD", "tags": ["graph"]}})
    );

    let credentials = Credentials::new("session-token", "csrf-token")?;
    let debug = format!("{credentials:?}");
    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("session-token") && !debug.contains("csrf-token"));
    for value in [
        "",
        "space token",
        "semicolon;token",
        "line\nbreak",
        &"a".repeat(4097),
    ] {
        assert!(matches!(
            Credentials::new(value, "csrf-token"),
            Err(Error::InvalidCredentials)
        ));
    }

    let auth_body = json!({"data": {"userStatus": {"isSignedIn": true}}}).to_string();
    let (result, request) = authenticated_exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{auth_body}",
            auth_body.len()
        ),
        "POST",
        "/graphql/",
        "https://leetcode.com/",
        |client, credentials| Box::pin(client.auth_status(credentials)),
    )
    .await?;
    assert!(result?);
    assert_eq!(request["operationName"], "userStatus");
    assert!(request.get("variables").is_none());
    assert!(
        request["query"]
            .as_str()
            .is_some_and(|query| query.contains("userStatus { isSignedIn username }"))
    );

    let starter_body = json!({"data": {"question": {
        "questionId": "1", "title": "Two Sum", "titleSlug": "two-sum",
        "codeDefinition": "[{\"value\":\"rust\",\"text\":\"Rust\",\"defaultCode\":\"impl Solution {}\"},{\"value\":\"python3\",\"text\":\"Python3\",\"defaultCode\":\"class Solution:\\n    pass\"}]",
        "isPaidOnly": false
    }}})
    .to_string();
    let (result, request, _) = exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{starter_body}",
            starter_body.len()
        ),
        |client, _| Box::pin(client.starter("two-sum", |_, _| {})),
    )
    .await?;
    let starter = result?;
    assert_eq!(starter.question_id.as_ref(), "1");
    assert_eq!(starter.id.as_ref(), "two-sum");
    assert_eq!(starter.title.as_ref(), "Two Sum");
    assert_eq!(starter.snippets.len(), 2);
    assert_eq!(starter.snippets[0].language.as_ref(), "Rust");
    assert_eq!(starter.snippets[0].language_slug.as_ref(), "rust");
    assert_eq!(starter.snippets[0].source.as_ref(), "impl Solution {}");
    assert_eq!(starter.snippets[1].language.as_ref(), "Python3");
    assert_eq!(starter.snippets[1].language_slug.as_ref(), "python3");
    assert_eq!(request["operationName"], "questionEditorData");
    assert_eq!(request["variables"], json!({"titleSlug": "two-sum"}));
    assert!(request["query"].as_str().is_some_and(|query| {
        query.contains("questionId title titleSlug codeDefinition isPaidOnly")
    }));

    let submit_body = json!({"submission_id": 42}).to_string();
    let (result, request) = authenticated_exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{submit_body}",
            submit_body.len()
        ),
        "POST",
        "/problems/two-sum/submit/",
        "https://leetcode.com/",
        |client, credentials| {
            Box::pin(client.submit(credentials, "two-sum", "1", "rust", "impl Solution {}"))
        },
    )
    .await?;
    assert_eq!(result?, 42);
    assert_eq!(
        request,
        json!({"question_id": "1", "lang": "rust", "typed_code": "impl Solution {}"})
    );

    let submission_body = json!({
        "state": "SUCCESS", "status_msg": "Accepted", "status_code": 10,
        "status_runtime": "0 ms", "status_memory": "2 MB", "memory": 2_000_000,
        "total_correct": 3, "total_testcases": 3,
        "full_compile_error": "", "full_runtime_error": ""
    })
    .to_string();
    let (result, request) = authenticated_exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{submission_body}",
            submission_body.len()
        ),
        "GET",
        "/submissions/detail/42/check/",
        "https://leetcode.com/",
        |client, credentials| Box::pin(client.submission(credentials, 42)),
    )
    .await?;
    assert_eq!(request, Value::Null);
    assert!(matches!(
        result?,
        SubmissionState::Complete(result)
            if result.id == 42
                && result.accepted
                && result.runtime.as_deref() == Some("0 ms")
                && result.memory.as_deref() == Some("2 MB")
                && result.passed == Some(3)
                && result.total == Some(3)
    ));

    let pending_body = json!({"state": "PENDING"}).to_string();
    let (result, _) = authenticated_exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{pending_body}",
            pending_body.len()
        ),
        "GET",
        "/submissions/detail/42/check/",
        "https://leetcode.com/",
        |client, credentials| Box::pin(client.submission(credentials, 42)),
    )
    .await?;
    assert!(matches!(result?, SubmissionState::Pending));

    let (result, _) = authenticated_exchange(
        "HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n".into(),
        "POST",
        "/graphql/",
        "https://leetcode.com/",
        |client, credentials| Box::pin(client.auth_status(credentials)),
    )
    .await?;
    assert!(matches!(result, Err(Error::Authentication)));

    let too_many_snippets = serde_json::to_string(
        &(0..65)
            .map(|index| {
                json!({"value": format!("x{index}"), "text": format!("X{index}"), "defaultCode": "x"})
            })
            .collect::<Vec<_>>(),
    )?;
    let oversized_label = serde_json::to_string(&json!([
        {"value": "rust", "text": "L".repeat(65), "defaultCode": "x"}
    ]))?;
    let oversized_slug = serde_json::to_string(&json!([
        {"value": "r".repeat(33), "text": "Rust", "defaultCode": "x"}
    ]))?;
    for (name, code_definition) in [
        ("empty", "[]".to_owned()),
        (
            "duplicate language",
            "[{\"value\":\"rust\",\"text\":\"Rust\",\"defaultCode\":\"x\"},{\"value\":\"rust\",\"text\":\"Rust 2\",\"defaultCode\":\"y\"}]".to_owned(),
        ),
        (
            "duplicate label",
            "[{\"value\":\"rust\",\"text\":\"Rust\",\"defaultCode\":\"x\"},{\"value\":\"python3\",\"text\":\"Rust\",\"defaultCode\":\"y\"}]".to_owned(),
        ),
        (
            "invalid language",
            "[{\"value\":\"Rust\",\"text\":\"Rust\",\"defaultCode\":\"x\"}]".to_owned(),
        ),
        ("oversized label", oversized_label),
        ("oversized slug", oversized_slug),
        (
            "empty source",
            "[{\"value\":\"rust\",\"text\":\"Rust\",\"defaultCode\":\"\"}]".to_owned(),
        ),
        ("too many", too_many_snippets),
    ] {
        let invalid_starter = json!({"data": {"question": {
            "questionId": "1", "title": "Two Sum", "titleSlug": "two-sum",
            "codeDefinition": code_definition, "isPaidOnly": false
        }}})
        .to_string();
        let (result, _, _) = exchange(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{invalid_starter}",
                invalid_starter.len()
            ),
            |client, _| Box::pin(client.starter("two-sum", |_, _| {})),
        )
        .await?;
        assert!(matches!(result, Err(Error::InvalidResponse)), "case={name}");
    }

    let invalid_submission =
        json!({"state": "SUCCESS", "status_msg": "Accepted", "status_code": 10,
        "total_correct": 2, "total_testcases": 1})
        .to_string();
    let (result, _) = authenticated_exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{invalid_submission}",
            invalid_submission.len()
        ),
        "GET",
        "/submissions/detail/42/check/",
        "https://leetcode.com/",
        |client, credentials| Box::pin(client.submission(credentials, 42)),
    )
    .await?;
    assert!(matches!(result, Err(Error::InvalidSubmissionResult)));

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

    for (name, body, expected) in [
        (
            "daily null data",
            json!({"data": null}),
            Failure::InvalidResponse,
        ),
        (
            "daily null challenge",
            json!({"data": {"activeDailyCodingChallengeQuestion": null}}),
            Failure::InvalidResponse,
        ),
        (
            "daily null question",
            json!({"data": {"activeDailyCodingChallengeQuestion": {"question": null}}}),
            Failure::InvalidResponse,
        ),
        (
            "daily invalid slug",
            json!({"data": {"activeDailyCodingChallengeQuestion": {"question": {
                "frontendQuestionId": "1", "titleSlug": "../today", "title": "Today",
                "content": "<p>Solve it.</p>", "isPaidOnly": false
            }}}}),
            Failure::InvalidResponse,
        ),
        (
            "daily graphql",
            json!({"data": null, "errors": [{"message": "sensitive server detail"}]}),
            Failure::Graphql,
        ),
    ] {
        let body = body.to_string();
        let (result, _, _) = exchange(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            ),
            |client, _| Box::pin(client.daily(|_, _| {})),
        )
        .await?;
        assert!(has_failure(&result, expected), "case={name}: {result:?}");
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

    for (name, number) in [
        ("missing", None),
        ("zero", Some("0")),
        ("non-numeric", Some("one")),
    ] {
        let mut body = problem_body();
        let question = body["data"]["question"]
            .as_object_mut()
            .ok_or("missing question")?;
        if let Some(number) = number {
            question.insert("frontendQuestionId".into(), json!(number));
        } else {
            question.remove("frontendQuestionId");
        }
        let result = query(body).await?;
        assert!(
            has_failure(&result, Failure::InvalidResponse),
            "number={name}"
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
    for tag in [
        "",
        "-graph",
        "graph-",
        "graph_tag",
        "graph tag",
        "graph\ntag",
        "café",
        &"a".repeat(65),
    ] {
        assert!(matches!(
            client.list(None, Some(tag), |_, _| {}).await,
            Err(Error::InvalidTag)
        ));
    }
    for source in ["", "impl Solution {\0}", &"a".repeat(1_048_577)] {
        assert!(matches!(
            client
                .submit(&credentials, "two-sum", "1", "rust", source)
                .await,
            Err(Error::InvalidSource)
        ));
    }
    for question_id in ["", "0", "one", "1\n", &"1".repeat(33)] {
        assert!(matches!(
            client
                .submit(
                    &credentials,
                    "two-sum",
                    question_id,
                    "rust",
                    "impl Solution {}",
                )
                .await,
            Err(Error::InvalidResponse)
        ));
    }
    for language in ["", "Rust", "rust-lang", "rust\n", &"r".repeat(33)] {
        assert!(matches!(
            client
                .submit(&credentials, "two-sum", "1", language, "impl Solution {}")
                .await,
            Err(Error::InvalidLanguage)
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

#[tokio::test]
async fn contest_contract() -> TestResult {
    let upcoming = json!({"data": {"upcomingContests": [
        {"title": "Weekly Contest 400", "titleSlug": "weekly-contest-400", "startTime": 1_700_000_000, "duration": 5_400, "isVirtual": false}
    ]}})
    .to_string();
    let (result, request, progress) = exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{upcoming}",
            upcoming.len()
        ),
        |client, progress| Box::pin(client.contests(|bytes, total| progress.push((bytes, total)))),
    )
    .await?;
    let contests = result?;
    assert_eq!(contests.len(), 1);
    assert_eq!(contests[0].slug.as_ref(), "weekly-contest-400");
    assert_eq!(contests[0].duration_seconds, 5_400);
    assert!(!contests[0].virtual_contest);
    assert_eq!(request["operationName"], "upcomingContests");
    assert!(request.get("variables").is_none());
    assert!(request["query"].as_str().is_some_and(|query| {
        query.contains("upcomingContests { title titleSlug startTime duration isVirtual }")
    }));
    assert_eq!(progress.first(), Some(&(0, Some(upcoming.len() as u64))));
    assert_eq!(
        progress.last(),
        Some(&(upcoming.len(), Some(upcoming.len() as u64)))
    );

    let contest = json!({"data": {"contest": {
        "title": "Weekly Contest 400", "titleSlug": "weekly-contest-400",
        "startTime": "1700000000", "duration": "5400", "isVirtual": true
    }}})
    .to_string();
    let (result, request, _) = exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{contest}",
            contest.len()
        ),
        |client, _| Box::pin(client.contest("weekly-contest-400", |_, _| {})),
    )
    .await?;
    let contest = result?;
    assert_eq!(contest.start_time, 1_700_000_000);
    assert_eq!(contest.duration_seconds, 5_400);
    assert!(contest.virtual_contest);
    assert_eq!(request["operationName"], "contest");
    assert_eq!(
        request["variables"],
        json!({"titleSlug": "weekly-contest-400"})
    );
    assert!(request["query"].as_str().is_some_and(|query| {
        query.contains("contest(titleSlug: $titleSlug)")
            && query.contains("title titleSlug startTime duration isVirtual")
    }));

    let registration = json!({"data": {"contest": {
        "titleSlug": "weekly-contest-400", "userRegistered": true
    }}})
    .to_string();
    let (result, request) = authenticated_exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{registration}",
            registration.len()
        ),
        "POST",
        "/graphql/",
        "https://leetcode.com/",
        |client, credentials| {
            Box::pin(client.contest_registration("weekly-contest-400", credentials, |_, _| {}))
        },
    )
    .await?;
    let registration = result?;
    assert_eq!(registration.slug.as_ref(), "weekly-contest-400");
    assert!(registration.registered);
    assert_eq!(request["operationName"], "contestRegistration");
    assert_eq!(
        request["variables"],
        json!({"titleSlug": "weekly-contest-400"})
    );
    assert!(request["query"].as_str().is_some_and(|query| {
        query.contains("contest(titleSlug: $titleSlug) { titleSlug userRegistered }")
    }));

    for (start_time, duration) in [(0, 5_400), (1_700_000_000, 604_801)] {
        let invalid = json!({"data": {"upcomingContests": [{
            "title": "Weekly Contest 400", "titleSlug": "weekly-contest-400",
            "startTime": start_time, "duration": duration, "isVirtual": false
        }]}})
        .to_string();
        let (result, _, _) = exchange(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{invalid}",
                invalid.len()
            ),
            |client, _| Box::pin(client.contests(|_, _| {})),
        )
        .await?;
        assert!(matches!(result, Err(Error::InvalidResponse)));
    }
    Ok(())
}

#[tokio::test]
async fn discussion_contract() -> TestResult {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let client = Client {
        http: http_builder()
            .https_only(false)
            .no_proxy()
            .timeout(Duration::from_secs(2))
            .build()?,
        endpoint: format!("http://{}/graphql/", listener.local_addr()?).into(),
    };
    let trending = json!({"data": {"cachedTrendingCategoryTopics": [{
        "id": 7, "title": "A useful pattern", "viewCount": 0, "topLevelCommentCount": "2",
        "post": {"voteCount": "3", "creationDate": 1700000000, "author": null}
    }]}})
    .to_string();
    let question =
        json!({"data": {"question": {"questionId": "1", "titleSlug": "two-sum"}}}).to_string();
    let problem = json!({"data": {"questionTopics": {
        "totalNum": "1", "data": [{
            "id": "8", "title": "Hash map", "viewCount": 4, "topLevelCommentCount": 1,
            "post": {"voteCount": 6, "creationDate": "1700000001", "author": {"username": "deleted_user"}}
        }]
    }}})
    .to_string();
    let detail = json!({"data": {"topic": {
        "id": "7", "title": "A useful pattern", "viewCount": "0", "topLevelCommentCount": 2,
        "tags": ["array"], "pinned": true,
        "post": {"voteCount": 3, "creationDate": "1700000000", "updationDate": null,
            "content": " article-topic ", "author": null}
    }}})
    .to_string();
    let article = json!({"data": {"ugcArticleDiscussionArticle": {
        "uuid": "5a5e3f62-4b78-4e3e-a3cc-4bba0c7617dc", "content": "Use a map."
    }}})
    .to_string();
    let invalid = json!({"data": {"cachedTrendingCategoryTopics": [{
        "id": 9, "title": "x".repeat(257), "viewCount": 0, "topLevelCommentCount": 0,
        "post": {"voteCount": 0, "creationDate": 1700000000, "author": null}
    }]}})
    .to_string();
    let responses = [trending, question, problem, detail, article, invalid];
    let server = async {
        let mut requests = Vec::new();
        for response in responses {
            let (mut stream, _) = listener.accept().await?;
            let mut request = Vec::new();
            let mut buffer = [0; 4096];
            let header_end = loop {
                let read = stream.read(&mut buffer).await?;
                assert_ne!(read, 0, "client closed before sending its request");
                request.extend_from_slice(&buffer[..read]);
                assert!(request.len() < 16 * 1024);
                if let Some(end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                    break end + 4;
                }
            };
            let headers = std::str::from_utf8(&request[..header_end - 4])?.to_ascii_lowercase();
            assert!(headers.starts_with("post /graphql/ http/1.1\r\n"));
            let length = headers
                .lines()
                .find_map(|line| line.strip_prefix("content-length: "))
                .ok_or("missing request content length")?
                .parse::<usize>()?;
            while request.len() < header_end + length {
                let read = stream.read(&mut buffer).await?;
                assert_ne!(read, 0, "client closed before sending its request body");
                request.extend_from_slice(&buffer[..read]);
            }
            requests.push(serde_json::from_slice(
                &request[header_end..header_end + length],
            )?);
            stream
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{response}",
                        response.len()
                    )
                    .as_bytes(),
                )
                .await?;
        }
        Ok::<Vec<Value>, Box<dyn std::error::Error>>(requests)
    };
    let work = async {
        let trending = client.trending_discussions(|_, _| {}).await?;
        assert_eq!(trending.total, None);
        assert_eq!(trending.discussions[0].author, None);
        assert_eq!(trending.discussions[0].comments, 2);
        let problem = client.problem_discussions("two-sum", |_, _| {}).await?;
        assert_eq!(problem.total, Some(1));
        assert_eq!(problem.discussions[0].id, 8);
        assert_eq!(problem.discussions[0].author, None);
        let detail = client.discussion(7, |_, _| {}).await?;
        assert_eq!(detail.author, None);
        assert_eq!(detail.tags, [Box::<str>::from("array")]);
        assert!(detail.pinned);
        assert_eq!(detail.content.as_ref(), "Use a map.");
        assert!(matches!(
            client.trending_discussions(|_, _| {}).await,
            Err(Error::InvalidResponse)
        ));
        Ok::<(), Error>(())
    };
    let (result, requests) =
        timeout(Duration::from_secs(3), async { tokio::join!(work, server) }).await?;
    result?;
    let requests = requests?;
    assert_eq!(
        requests
            .iter()
            .map(|request| request["operationName"].as_str())
            .collect::<Vec<_>>(),
        [
            Some("trendingDiscussions"),
            Some("discussionQuestion"),
            Some("problemDiscussions"),
            Some("discussion"),
            Some("discussionArticle"),
            Some("trendingDiscussions")
        ]
    );
    assert_eq!(requests[1]["variables"], json!({"titleSlug": "two-sum"}));
    assert_eq!(
        requests[2]["variables"],
        json!({"questionId": 1, "orderBy": "newest_to_oldest", "pageNo": 1, "numPerPage": 20})
    );
    assert_eq!(requests[3]["variables"], json!({"topicId": 7}));
    assert_eq!(requests[4]["variables"], json!({"topicId": 7}));
    assert!(requests[2]["query"].as_str().is_some_and(|query| {
        query.contains("$orderBy: TopicSortingOption!")
            && query.contains("$pageNo: Int!")
            && query.contains("$numPerPage: Int!")
    }));
    assert!(requests[4]["query"].as_str().is_some_and(|query| {
        query.contains("$topicId: ID")
            && query.contains("ugcArticleDiscussionArticle(topicId: $topicId)")
    }));
    Ok(())
}

#[tokio::test]
async fn account_stats_contract() -> TestResult {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let client = Client {
        http: http_builder()
            .https_only(false)
            .no_proxy()
            .timeout(Duration::from_secs(2))
            .build()?,
        endpoint: format!("http://{}/graphql/", listener.local_addr()?).into(),
    };
    let credentials = Credentials::new("session-token", "csrf-token")?;
    let status =
        json!({"data": {"userStatus": {"isSignedIn": true, "username": "possum"}}}).to_string();
    let stats = json!({"data": {
        "matchedUser": {
            "username": "possum",
            "submitStats": {
                "acSubmissionNum": [
                    {"difficulty": "All", "count": 10, "submissions": 13},
                    {"difficulty": "Easy", "count": 4, "submissions": 5},
                    {"difficulty": "Medium", "count": 5, "submissions": 7},
                    {"difficulty": "Hard", "count": 1, "submissions": 1}
                ],
                "totalSubmissionNum": [
                    {"difficulty": "All", "count": 20, "submissions": 28},
                    {"difficulty": "Easy", "count": 8, "submissions": 10},
                    {"difficulty": "Medium", "count": 10, "submissions": 15},
                    {"difficulty": "Hard", "count": 2, "submissions": 3}
                ]
            }
        },
        "submissionList": {"hasNext": true, "submissions": [
            {
                "id": 42, "statusDisplay": "Accepted", "title": "Two Sum", "titleSlug": "two-sum",
                "timestamp": 1700000000, "lang": "Rust", "runtime": "4 ms", "memory": "16 MB",
                "url": "/submissions/detail/42/"
            },
            {
                "id": "43", "statusDisplay": "Wrong Answer", "title": "Add Two Numbers",
                "titleSlug": "add-two-numbers", "timestamp": "1700000001", "lang": "C++",
                "runtime": null, "memory": null, "url": null
            }
        ]}
    }})
    .to_string();
    let responses = [status, stats];
    let server = async {
        let mut requests = Vec::new();
        for response in responses {
            let (mut stream, _) = listener.accept().await?;
            let mut request = Vec::new();
            let mut buffer = [0; 4096];
            let header_end = loop {
                let read = stream.read(&mut buffer).await?;
                assert_ne!(read, 0, "client closed before sending its request");
                request.extend_from_slice(&buffer[..read]);
                assert!(request.len() < 16 * 1024);
                if let Some(end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                    break end + 4;
                }
            };
            let headers = std::str::from_utf8(&request[..header_end - 4])?.to_ascii_lowercase();
            assert!(headers.starts_with("post /graphql/ http/1.1\r\n"));
            for expected in [
                "cookie: leetcode_session=session-token; csrftoken=csrf-token",
                "x-csrftoken: csrf-token",
                "origin: https://leetcode.com",
            ] {
                assert!(headers.lines().any(|line| line.starts_with(expected)));
            }
            let length = headers
                .lines()
                .find_map(|line| line.strip_prefix("content-length: "))
                .ok_or("missing request content length")?
                .parse::<usize>()?;
            while request.len() < header_end + length {
                let read = stream.read(&mut buffer).await?;
                assert_ne!(read, 0, "client closed before sending its request body");
                request.extend_from_slice(&buffer[..read]);
            }
            requests.push(serde_json::from_slice(
                &request[header_end..header_end + length],
            )?);
            stream
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{response}",
                        response.len()
                    )
                    .as_bytes(),
                )
                .await?;
        }
        Ok::<Vec<Value>, Box<dyn std::error::Error>>(requests)
    };
    let (result, requests) = timeout(Duration::from_secs(3), async {
        tokio::join!(client.account_stats(&credentials), server)
    })
    .await?;
    let stats = result?;
    let requests = requests?;
    assert_eq!(stats.username.as_ref(), "possum");
    assert_eq!(stats.solved.all, 10);
    assert_eq!(stats.accepted_submissions.medium, 7);
    assert_eq!(stats.submissions.hard, 3);
    assert_eq!(stats.recent_submissions[0].id, 42);
    assert_eq!(stats.recent_submissions[0].slug.as_ref(), "two-sum");
    assert_eq!(stats.recent_submissions[1].id, 43);
    assert_eq!(stats.recent_submissions[1].timestamp, 1_700_000_001);
    assert!(stats.has_more_submissions);
    assert_eq!(requests[0]["operationName"], "userStatus");
    assert_eq!(requests[1]["operationName"], "accountStats");
    assert_eq!(requests[1]["variables"], json!({"username": "possum"}));
    assert!(requests[1]["query"].as_str().is_some_and(|query| {
        query.contains("matchedUser(username: $username)")
            && query.contains("submissionList(offset: 0, limit: 10)")
            && query.contains("statusDisplay")
    }));
    Ok(())
}

#[tokio::test]
async fn remote_run_contract() -> TestResult {
    let cases_body = json!({"data": {"question": {
        "questionId": "1", "titleSlug": "two-sum", "isPaidOnly": false,
        "enableRunCode": true, "exampleTestcaseList": ["[2,7,11,15]\n9", "", "[3,3]\n6"],
        "sampleTestCase": "ignored"
    }}})
    .to_string();
    let (result, request, _) = exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{cases_body}",
            cases_body.len()
        ),
        |client, _| Box::pin(client.test_cases("two-sum", |_, _| {})),
    )
    .await?;
    let cases = result?;
    assert_eq!(cases.question_id.as_ref(), "1");
    assert_eq!(cases.input.as_ref(), "[2,7,11,15]\n9\n[3,3]\n6");
    assert_eq!(request["operationName"], "consolePanelConfig");
    assert_eq!(request["variables"], json!({"titleSlug": "two-sum"}));
    assert!(request["query"].as_str().is_some_and(|query| {
        query.contains(
            "questionId titleSlug isPaidOnly enableRunCode exampleTestcaseList sampleTestCase",
        )
    }));

    let fallback_body = json!({"data": {"question": {
        "questionId": "1", "titleSlug": "two-sum", "isPaidOnly": false,
        "enableRunCode": true, "exampleTestcaseList": [], "sampleTestCase": "[3,2,4]\n6"
    }}})
    .to_string();
    let (result, _, _) = exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{fallback_body}",
            fallback_body.len()
        ),
        |client, _| Box::pin(client.test_cases("two-sum", |_, _| {})),
    )
    .await?;
    assert_eq!(result?.input.as_ref(), "[3,2,4]\n6");

    let run_body = json!({"interpret_id": "runcode_1729.42_ab-CD"}).to_string();
    let (result, request) = authenticated_exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{run_body}",
            run_body.len()
        ),
        "POST",
        "/problems/two-sum/interpret_solution/",
        "https://leetcode.com/problems/two-sum/",
        |client, credentials| {
            Box::pin(client.run(
                credentials,
                "two-sum",
                "1",
                "rust",
                "impl Solution {}",
                "[2,7]\n9",
            ))
        },
    )
    .await?;
    assert_eq!(result?.as_ref(), "runcode_1729.42_ab-CD");
    assert_eq!(
        request,
        json!({
            "data_input": "[2,7]\n9", "lang": "rust", "question_id": "1",
            "typed_code": "impl Solution {}"
        })
    );

    let rejected_body = json!({"detail": "Session expired"}).to_string();
    let (result, _) = authenticated_exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{rejected_body}",
            rejected_body.len()
        ),
        "POST",
        "/problems/two-sum/interpret_solution/",
        "https://leetcode.com/problems/two-sum/",
        |client, credentials| {
            Box::pin(client.run(
                credentials,
                "two-sum",
                "1",
                "rust",
                "impl Solution {}",
                "[2,7]\n9",
            ))
        },
    )
    .await?;
    assert!(matches!(
        result,
        Err(Error::RunRejected(message)) if message.as_ref() == "Session expired"
    ));

    let pending_body = json!({"state": "STARTED"}).to_string();
    let (result, request) = authenticated_exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{pending_body}",
            pending_body.len()
        ),
        "GET",
        "/submissions/detail/runcode_1729.42_ab-CD/check/",
        "https://leetcode.com/",
        |client, credentials| Box::pin(client.run_result(credentials, "runcode_1729.42_ab-CD")),
    )
    .await?;
    assert_eq!(request, Value::Null);
    assert!(matches!(result?, RunState::Pending));

    let success_body = json!({
        "state": "SUCCESS", "status_msg": null, "status_code": null,
        "correct_answer": false,
        "status_runtime": "3 ms", "status_memory": "1.9 MB",
        "total_correct": null, "total_testcases": 2,
        "code_answer": ["[0,1]", "[0,0]"],
        "expected_code_answer": ["[0,1]", "[1,2]"],
        "code_output": ["ignored"], "std_output_list": ["", "trace"],
        "input_formatted": "[2,7]\n9\u{001b}[31m",
        "full_compile_error": "", "full_runtime_error": ""
    })
    .to_string();
    let (result, _) = authenticated_exchange(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{success_body}",
            success_body.len()
        ),
        "GET",
        "/submissions/detail/runcode_1729.42_ab-CD/check/",
        "https://leetcode.com/",
        |client, credentials| Box::pin(client.run_result(credentials, "runcode_1729.42_ab-CD")),
    )
    .await?;
    let result = result?;
    assert!(
        matches!(
            &result,
            RunState::Complete(result)
                if result.id.as_ref() == "runcode_1729.42_ab-CD"
                    && !result.passed
                    && result.status.as_ref() == "Wrong Answer"
                    && result.passed_cases == Some(1)
                    && result.total_cases == Some(2)
                    && result.input.as_deref() == Some("[2,7]\n9[31m")
                    && result.output.as_deref() == Some("[0,1]\n[0,0]")
                    && result.expected.as_deref() == Some("[0,1]\n[1,2]")
                    && result.message.is_none()
        ),
        "{result:?}"
    );

    for (name, body, expected) in [
        (
            "run disabled",
            json!({"data": {"question": {
                "questionId": "1", "titleSlug": "two-sum", "isPaidOnly": false,
                "enableRunCode": false, "exampleTestcaseList": ["x"], "sampleTestCase": "x"
            }}}),
            "unavailable",
        ),
        (
            "no cases",
            json!({"data": {"question": {
                "questionId": "1", "titleSlug": "two-sum", "isPaidOnly": false,
                "enableRunCode": true, "exampleTestcaseList": ["", " "], "sampleTestCase": "must not fall back"
            }}}),
            "invalid",
        ),
        (
            "too many cases",
            json!({"data": {"question": {
                "questionId": "1", "titleSlug": "two-sum", "isPaidOnly": false,
                "enableRunCode": true, "exampleTestcaseList": vec!["x"; 129], "sampleTestCase": "x"
            }}}),
            "invalid",
        ),
    ] {
        let body = body.to_string();
        let (result, _, _) = exchange(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            ),
            |client, _| Box::pin(client.test_cases("two-sum", |_, _| {})),
        )
        .await?;
        assert!(
            matches!(
                (result, expected),
                (Err(Error::RunUnavailable), "unavailable")
                    | (Err(Error::InvalidTestCases), "invalid")
            ),
            "case={name}"
        );
    }

    let client = Client::new()?;
    let credentials = Credentials::new("session-token", "csrf-token")?;
    for input in ["", "bad\0input", &"x".repeat(1_048_577)] {
        assert!(matches!(
            client
                .run(
                    &credentials,
                    "two-sum",
                    "1",
                    "rust",
                    "impl Solution {}",
                    input,
                )
                .await,
            Err(Error::InvalidTestInput)
        ));
    }
    for id in ["", "../run", "run/42", "run\n42", &"r".repeat(129)] {
        assert!(matches!(
            client.run_result(&credentials, id).await,
            Err(Error::InvalidResponse)
        ));
    }
    Ok(())
}
