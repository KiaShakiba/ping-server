# ping-server
The goal of this repository is to compare the efficiency of a ping server built with Rust std::net and tokio::net.

On Linux (Ubuntu 24), the std::net implementation is consistently faster than the Tokio implementation, especially as the number of concurrent clients increases.

This repository contains a simple experiment to test the rate at which a TCP server written using either std::net or tokio::net can respond to 1,000,000 pings sent from `N` concurrent clients (where each client sends 1,000,000/`N` pings).

Steps for measuring this locally:
1. Run the Tokio server with `cargo run -r --bin tokio-server`
2. Run the benchmark with `cargo run -r --bin benchmark -- -c 4` (this runs the benchmark with 4 concurrent clients)
3. Wait for the benchmark to complete and note the results.
4. Kill the Tokio server and run the std::net server with `cargo run -r --bin std-server`
5. Re-run the benchmark with the same command as in step 2.
