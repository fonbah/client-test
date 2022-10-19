use std::time::{Duration, Instant};

use tokio::net::TcpStream;
use hyper::{client::conn, Body};
use tower::ServiceExt;
use http::{Request, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let n: u8 = match args.first() {
        Some(_n) => match _n.parse() {
            Ok(n) if n > 0 && n <= 100 => n,
            _ => panic!("Error: n must be from 1 to 100!"),
        },
        _ => panic!("Error: n param required!"),
    };

    let mut tm_max = 0;

    let now = Instant::now();

    let mut stream = TcpStream::connect("127.0.0.1:3000").await?;

    let (mut request_sender, connection) = conn::Builder::new()
        .http2_only(true)
        .http2_keep_alive_timeout(Duration::new(2, 0))
        .handshake( stream).await?;

    // checking connection
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            println!("Error in connection: {}", e);
        }
    });

    let mut reqs = Vec::with_capacity(n as usize);
    let mut resp_futes = Vec::with_capacity(n as usize);
    let mut results = Vec::with_capacity(n as usize);

    //requests preparing
    for _n in 0..n {
        let rq = Request::builder()
            .header("Host", "localhost")
            .method("GET")
            .body(Body::from(""))?;
        reqs.push(rq);
    }

    // send all requests
    for req in reqs {
        request_sender.ready().await?;
        let resp = request_sender.send_request(req);
        resp_futes.push(resp);
    }

    // async responce handlers
    let resp_handlers = resp_futes.into_iter().map(move |resp| {
        tokio::spawn(async move { resp.await })
    }).collect::<Vec<_>>();

    // responce handlers join
    for handler in resp_handlers {
        let start = Instant::now();
        let result = handler.await?;
        let tm = start.elapsed().as_millis();
        if tm > tm_max { tm_max = tm }
        results.push(result);
    }

    let req_errs = results.iter().filter(|r| r.is_err()).count();

    println!("responces received {:?}", results.len());
    println!("responces errors {:?}", req_errs);

    println!("responce time max {:?} ms", tm_max);
    println!("elapsed total {:?} ms", now.elapsed().as_millis());

    Ok(())
}
