/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::{
	thread,
	io::{Read, Write},
	time::{Instant, Duration},
	net::TcpStream,
};

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[arg(short, long, default_value_t = 1)]
	clients: u32,
}

const TOTAL_NUM_PINGS: usize = 1_000_000;

fn main() -> anyhow::Result<()> {
	let args = Args::parse();
	let mut handles = Vec::new();

	let client_num_pings = TOTAL_NUM_PINGS / args.clients as usize;

	for i in 0..args.clients {
		println!("Running client {i} with {client_num_pings} pings...");

		let handle = thread::spawn(move || {
			send_pings(client_num_pings)
				.expect("An error occurred sending pings")
		});

		handles.push(handle);
	}

	println!();

	let mut total_duration = Duration::ZERO;

	for (index, handle) in handles.into_iter().enumerate() {
		let client_duration = handle.join().unwrap();
		let rate = client_num_pings as f64 / client_duration.as_secs_f64();

		println!("Client {index}: {rate:.2} pings/sec");

		total_duration += client_duration;
	}

	let rate = TOTAL_NUM_PINGS as f64 / total_duration.as_secs_f64();
	println!("\nAverage: {rate:.2} pings/sec");

	Ok(())
}

fn send_pings(num_pings: usize) -> anyhow::Result<Duration> {
	let mut stream = TcpStream::connect("localhost:3000")?;
	stream.set_nodelay(true)?;

	let mut ping_res = vec![0u8; 1 + 4 + 4];
	let mut total_duration = Duration::ZERO;

	for _ in 0..num_pings {
		let start = Instant::now();

		stream.write_all(&[0])?;
		stream.read_exact(&mut ping_res)?;

		total_duration += start.elapsed();
	}

	Ok(total_duration)
}
