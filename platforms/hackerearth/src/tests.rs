use std::{io, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    time::timeout,
};

use crate::{Client, Difficulty, Error, PracticePage, Problem, client::http_builder};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

async fn exchange<T>(
    responses: Vec<String>,
    operation: impl for<'a> FnOnce(
        &'a Client,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<T, Error>> + 'a>,
    >,
) -> TestResult<(Result<T, Error>, Vec<String>)> {
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
        let mut requests = Vec::with_capacity(responses.len());
        for response in responses {
            let (mut stream, _) = listener.accept().await?;
            let mut request = Vec::new();
            let mut buffer = [0; 1024];
            loop {
                let read = stream.read(&mut buffer).await?;
                assert_ne!(read, 0, "client closed before sending its request");
                request.extend_from_slice(&buffer[..read]);
                assert!(request.len() < 16 * 1024);
                if request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                    break;
                }
            }
            let headers = std::str::from_utf8(&request)?;
            assert!(!headers.to_ascii_lowercase().contains("\r\ncookie:"));
            requests.push(headers.lines().next().unwrap_or_default().to_owned());
            if let Err(error) = stream.write_all(response.as_bytes()).await
                && !matches!(
                    error.kind(),
                    io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset
                )
            {
                return Err(error.into());
            }
        }
        Ok::<_, Box<dyn std::error::Error>>(requests)
    };
    let (result, requests) = timeout(Duration::from_secs(3), async {
        tokio::join!(operation(&client), server)
    })
    .await?;
    Ok((result, requests?))
}

fn response(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
}

#[tokio::test]
async fn public_practice_contract() -> TestResult {
    let list = r#"
<ul class="prob-list" id="prob-list">
  <li class="prob "><h4><a class="dark" href="/problem/algorithm/make-an-array-85abd7ad/">Make an array</a></h4><p>ATTEMPTED BY: <b>12</b> SUCCESS RATE: <b>56%</b> LEVEL: <b>Easy</b></p></li>
</ul>"#;
    let problem = r#"<script>var initial_state = { problemData: {"title":"Make an array","description":"<p>Use $A$.</p>","tags":"Math,Algorithms","level":"E","success_rate":56,"points":20,"attempted_by":12,"time_limit":1.0,"memory_limit":256} };</script>"#;
    let (result, requests) = exchange(vec![response(list), response(problem)], |client| {
        Box::pin(async move {
            let page: PracticePage = client
                .practice("algorithms/searching/linear-search", 2, |_, _| {})
                .await?;
            assert_eq!(page.problems.len(), 1);
            assert_eq!(page.problems[0].slug.as_ref(), "make-an-array-85abd7ad");
            assert_eq!(page.problems[0].difficulty, Difficulty::Easy);
            assert_eq!(page.problems[0].success_rate, Some(56));
            let problem: Problem = client.problem("make-an-array-85abd7ad", |_, _| {}).await?;
            assert_eq!(problem.summary.title.as_ref(), "Make an array");
            assert_eq!(problem.statement_html.as_ref(), "<p>Use $A$.</p>");
            assert_eq!(problem.tags.len(), 2);
            assert_eq!(problem.time_limit_seconds, Some(1));
            Ok(())
        })
    })
    .await?;
    result?;
    assert_eq!(
        requests,
        [
            "GET /practice/algorithms/searching/linear-search/practice-problems/2/ HTTP/1.1",
            "GET /problem/algorithm/make-an-array-85abd7ad/ HTTP/1.1",
        ]
    );
    Ok(())
}

#[tokio::test]
async fn rejects_untrusted_selectors_and_missing_public_data() -> TestResult {
    let client = Client::new()?;
    assert!(matches!(
        client.practice("algorithms/../private", 1, |_, _| {}).await,
        Err(Error::InvalidTopic)
    ));
    assert!(matches!(
        client.problem("Not-a-slug", |_, _| {}).await,
        Err(Error::InvalidSlug)
    ));
    let (result, _) = exchange(vec![response("<html>missing</html>")], |client| {
        Box::pin(async move { client.problem("make-an-array-85abd7ad", |_, _| {}).await })
    })
    .await?;
    assert!(matches!(result, Err(Error::NotFound)));
    Ok(())
}
