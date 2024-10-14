use std::{
    hint::black_box,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::{Duration, Instant},
};

use criterion::{measurement::WallTime, BenchmarkGroup, BenchmarkId};
use socket2::SockRef;

pub fn bench_std_server(group: &mut BenchmarkGroup<'_, WallTime>) {
    let listener = TcpListener::bind("localhost:0").expect("failed to bind to address");
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || start_server(listener));

    for connection_count in [1u64, 2, 4, 8, 16, 32, 64] {
        group.bench_with_input(
            BenchmarkId::new("std", connection_count),
            &connection_count,
            |b, connection_count| {
                b.iter_custom(|iters| {
                    start_clients(port, black_box(*connection_count as usize), iters)
                });
            },
        );
    }
}

/// Start a server that listens for incoming connections and spawns a new thread for each connection.
fn start_server(listener: TcpListener) {
    thread::scope(|scope| {
        for stream in listener.incoming() {
            let stream = stream.expect("failed to accept incoming connection");
            scope.spawn(move || handle_stream(stream));
        }
    });
}

fn handle_stream(mut stream: TcpStream) -> io::Result<()> {
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
            .is_err_and(|e| e.kind() == io::ErrorKind::UnexpectedEof)
        {
            break Ok(());
        }
        stream.write_all(&ping_res)?;
    }
}

/// Start `connection_count` clients that connect to the server at `port` and send `iters` pings.
/// (iters / connection_count) pings will be sent by each client.
fn start_clients(port: u16, connection_count: usize, iters: u64) -> Duration {
    let client_num_pings = iters / connection_count as u64;
    let mut handles = Vec::new();
    for _ in 0..connection_count {
        let handle = thread::spawn(move || {
            send_pings(port, client_num_pings).expect("sending pings failed")
        });
        handles.push(handle);
    }
    handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .sum::<Duration>()
        / connection_count as u32
}

fn send_pings(port: u16, num_pings: u64) -> io::Result<Duration> {
    let addr = format!("localhost:{}", port);
    let mut stream = TcpStream::connect(addr)?;
    stream.set_nodelay(true)?;

    let mut ping_res = vec![0u8; 1 + 4 + 4];

    let start = Instant::now();
    for _ in 0..num_pings {
        stream.write_all(&[0])?;
        stream.read_exact(&mut ping_res)?;
    }
    Ok(start.elapsed())
}
