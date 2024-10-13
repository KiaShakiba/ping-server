/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use tokio::{
	net::{TcpListener, TcpStream},
	io::{AsyncReadExt, AsyncWriteExt},
};

use socket2::SockRef;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let listener = TcpListener::bind("localhost:3000").await?;

	println!("Tokio server listening on port 3000...");

	loop {
		let (stream, _) = listener.accept().await?;
		println!("Handling new connection...");

		tokio::spawn(async move {
			let _ = handle_stream(stream).await;
		});
	}
}

async fn handle_stream(mut stream: TcpStream) -> anyhow::Result<()> {
	let socket_ref = SockRef::from(&stream);

	socket_ref.set_nodelay(true)?;
	socket_ref.set_quickack(true)?;

	let mut ping_req = vec![0u8; 1];
	let mut ping_res = Vec::<u8>::new();

	ping_res.push(b"+"[0]);
	ping_res.extend_from_slice(&4u32.to_le_bytes());
	ping_res.extend_from_slice(b"pong");

	loop {
		stream.read_exact(&mut ping_req).await?;
		stream.write_all(&ping_res).await?;
	}
}
