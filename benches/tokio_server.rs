use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use criterion::{measurement::WallTime, BenchmarkGroup, BenchmarkId};
use socket2::SockRef;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

pub fn bench_tokio_server(group: &mut BenchmarkGroup<'_, WallTime>) {
    for connection_count in [1u64, 2, 4, 8, 16, 32, 64] {
        group.bench_with_input(
            BenchmarkId::new("tokio", connection_count),
            &connection_count,
            |b, connection_count| {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                b.to_async(runtime).iter_custom(|iters| async move {
                    let listener = TcpListener::bind("localhost:0")
                        .await
                        .expect("failed to bind to address");
                    let port = listener.local_addr().unwrap().port();
                    tokio::spawn(start_server(listener));
                    start_clients(port, black_box(*connection_count as usize), iters).await
                });
            },
        );
    }
}

/// Start a server that listens for incoming connections and spawns a new thread for each connection.
async fn start_server(listener: TcpListener) {
    loop {
        let (stream, _) = listener
            .accept()
            .await
            .expect("failed to accept incoming connection");
        tokio::spawn(async move {
            let _ = handle_stream(stream).await;
        });
    }
}

async fn handle_stream(mut stream: TcpStream) -> io::Result<()> {
    let socket_ref = SockRef::from(&stream);

    socket_ref.set_nodelay(true)?;
    // socket_ref.set_quickack(true)?;

    let mut ping_req = vec![0u8; 1];
    let mut ping_res = Vec::<u8>::new();

    ping_res.push(b"+"[0]);
    ping_res.extend_from_slice(&4u32.to_le_bytes());
    ping_res.extend_from_slice(b"pong");

    loop {
        if stream
            .read_exact(&mut ping_req)
            .await
            .is_err_and(|e| e.kind() == io::ErrorKind::UnexpectedEof)
        {
            break Ok(());
        }
        stream.write_all(&ping_res).await?;
    }
}

/// Start `connection_count` clients that connect to the server at `port` and send `iters` pings.
/// (iters / connection_count) pings will be sent by each client.
async fn start_clients(port: u16, connection_count: usize, iters: u64) -> Duration {
    let client_num_pings = iters / connection_count as u64;
    let mut handles = Vec::new();
    for _ in 0..connection_count {
        let handle = tokio::spawn(async move {
            send_pings(port, client_num_pings)
                .await
                .expect("sending pings failed")
        });
        handles.push(handle);
    }
    let mut total_duration = Duration::ZERO;
    for handle in handles {
        total_duration += handle.await.unwrap();
    }
    total_duration
}

async fn send_pings(port: u16, num_pings: u64) -> io::Result<Duration> {
    let addr = format!("localhost:{}", port);
    let mut stream = TcpStream::connect(addr).await?;
    stream.set_nodelay(true)?;

    let mut ping_res = vec![0u8; 1 + 4 + 4];

    let start = Instant::now();
    for _ in 0..num_pings {
        stream.write_all(&[0]).await?;
        stream.read_exact(&mut ping_res).await?;
    }
    Ok(start.elapsed())
}
