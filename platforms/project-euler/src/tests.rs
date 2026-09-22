use std::{io, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    time::timeout,
};

use crate::{
    ArchivePage, Client, Error, Problem, ProblemCatalog, RecentProblems, client::http_builder,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy)]
enum Operation {
    Archive,
    Catalog,
    Recent,
    MissingProblem,
    Problem,
}

async fn exchange(
    operation: Operation,
    responses: Vec<String>,
) -> TestResult<(Result<(), Error>, Vec<String>)> {
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
                if request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                    break;
                }
            }
            let line = std::str::from_utf8(&request)?
                .lines()
                .next()
                .unwrap_or_default()
                .to_owned();
            if let Err(error) = stream.write_all(response.as_bytes()).await
                && !matches!(
                    error.kind(),
                    io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset
                )
            {
                return Err(error.into());
            }
            requests.push(line);
        }
        Ok::<_, Box<dyn std::error::Error>>(requests)
    };
    let operation_result = async {
        match operation {
            Operation::Archive => {
                let page: ArchivePage = client.archive(1, |_, _| {}).await?;
                assert_eq!(page.problems.len(), 50);
                assert_eq!(page.problems[0].title.as_ref(), "Problem 1");
                assert_eq!(page.problems[0].published_at.as_deref(), Some("today"));
                assert_eq!(page.problems[49].number, 50);
            }
            Operation::Catalog => {
                let catalog: ProblemCatalog = client.catalog(|_, _| {}).await?;
                assert_eq!(catalog.problems.len(), 3);
                assert_eq!(catalog.problems[0].title.as_ref(), "Multiples of 3 or 5");
                assert_eq!(
                    catalog.problems[1].title.as_ref(),
                    "$N$th Digit of Reciprocals"
                );
                assert_eq!(catalog.problems[2].number, 1001);
                assert_eq!(catalog.problems[2].solved_count, Some(202));
            }
            Operation::Recent => {
                let recent: RecentProblems = client.recent(|_, _| {}).await?;
                assert_eq!(recent.problems.len(), 10);
                assert_eq!(recent.problems[0].title.as_ref(), "Problem 50");
                assert_eq!(recent.problems[9].number, 41);
            }
            Operation::Problem => {
                let problem: Problem = client.problem(1, |_, _| {}).await?;
                assert_eq!(problem.summary.title.as_ref(), "Multiples of 3 or 5");
                assert_eq!(problem.summary.published_at.as_deref(), Some("Friday"));
                assert_eq!(problem.difficulty.map(|value| value.level), Some(0));
                assert_eq!(problem.statement_html.as_ref(), "<p>Find $x$.</p>");
                assert_eq!(problem.license, "CC BY-NC-SA 4.0");
            }
            Operation::MissingProblem => {
                assert!(matches!(
                    client.problem(9_999, |_, _| {}).await,
                    Err(Error::NotFound)
                ));
            }
        }
        Ok(())
    };
    let (result, requests) = timeout(Duration::from_secs(3), async {
        tokio::join!(operation_result, server)
    })
    .await?;
    Ok((result, requests?))
}

#[tokio::test]
async fn public_html_contract() -> TestResult {
    let rows = (1..=50)
        .map(|number| format!(
            "<tr><td class=\"id_column\">{number}</td><td><a href=\"problem={number}\" title=\"Published on today\">Problem {number}</a></td><td><div class=\"center\">{number}</div></td></tr>"
        ))
        .collect::<String>();
    let archive = format!("<table id=\"problems_table\">{rows}</table>");
    let recent_rows = (41..=50)
        .rev()
        .map(|number| format!(
            "<tr><td class=\"id_column\">{number}</td><td><a href=\"problem={number}\" title=\"Published on today\">Problem {number}</a></td><td><div class=\"center\">{number}</div></td></tr>"
        ))
        .collect::<String>();
    let recent = format!("<table id=\"problems_table\">{recent_rows}</table>");
    let page = "<div id=\"content\"><h2>Multiples of 3 or 5</h2><span class=\"tooltiptext_right\">Published on Friday<br>Difficulty: Level 0 [1%]</span><h3>Problem 1</h3></div>";
    let minimal = "<p>Find $x$.</p>";
    let catalog = "ID##Title##Published##Solved By\r\n1##Multiples of 3 or 5##2001-10-05 17:00:00##856188\r\n820##$N$<sup>th</sup> Digit of Reciprocals##2022-12-10 16:00:00##1648\r\n1001##Connections I##2026-09-05 13:00:00##202\r\n";
    let cases = [
        (
            Operation::Archive,
            vec![archive],
            vec!["GET /archives;page=1 HTTP/1.1"],
        ),
        (
            Operation::Recent,
            vec![recent],
            vec!["GET /recent HTTP/1.1"],
        ),
        (
            Operation::Catalog,
            vec![catalog.into()],
            vec!["GET /minimal=problems HTTP/1.1"],
        ),
        (
            Operation::Problem,
            vec![page.into(), minimal.into()],
            vec!["GET /problem=1 HTTP/1.1", "GET /minimal=1 HTTP/1.1"],
        ),
        (
            Operation::MissingProblem,
            vec!["Data for that problem cannot be found".into()],
            vec!["GET /problem=9999 HTTP/1.1"],
        ),
    ];
    for (operation, bodies, expected_requests) in cases {
        let responses = bodies
            .into_iter()
            .map(|body| {
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                    body.len()
                )
            })
            .collect();
        let (result, requests) = exchange(operation, responses).await?;
        result?;
        assert_eq!(requests, expected_requests);
    }
    Ok(())
}
