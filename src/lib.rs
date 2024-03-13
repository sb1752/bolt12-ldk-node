use std::io;
use std::io::prelude::*;
use std::sync::Arc;
use ldk_node::{Builder, Config, Node, default_config};
use ldk_node::bitcoin::Network;

pub fn run() -> () {
    let mut config = default_config();
    config.network = Network::Signet;

    let mut builder = Builder::from_config(config);
    builder.set_esplora_server("https://mutinynet.com/api/".to_string());
    let node = Arc::new(builder.build().unwrap());

    node.start().unwrap();
    let event_node = Arc::clone(&node);
    std::thread::spawn(move || loop {
        let event = event_node.wait_next_event();
        println!("GOT NEW EVENT: {:?}", event);
        println!("Channels: {:?}", event_node.list_channels());
        println!("Payments: {:?}", event_node.list_payments());
        event_node.event_handled();
    });

    println!("Node ID: {}", node.node_id());
    println!("Address: {}", node.onchain_payment().new_address().unwrap());
    println!("Channels: {:?}", node.list_channels());
    println!("Payments: {:?}", node.list_payments());
    println!("Funds: {:?}", node.list_balances());

    let node_id = "02465ed5be53d04fde66c9418ff14a5f2267723810176c9212b722e542dc1afb1b".parse().unwrap();
    let address = "45.79.52.207:9735".parse().unwrap();
    node.connect_open_channel(node_id, address, 5_000_000, None, None, false).unwrap();

    node.sync_wallets().unwrap();

    node.spontaneous_payment().send(500_000, node_id).unwrap();

    pause();
    node.stop().unwrap();
}

fn pause() {
	let mut stdin = io::stdin();
	let mut stdout = io::stdout();

	// We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
	write!(stdout, "Press any key to continue...").unwrap();
	stdout.flush().unwrap();

	// Read a single byte and discard
	let _ = stdin.read(&mut [0u8]).unwrap();
}
