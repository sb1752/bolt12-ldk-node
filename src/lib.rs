use std::io;
use std::io::prelude::*;
use ldk_node::lightning::ln::msgs::SocketAddress;
use std::sync::Arc;
use ldk_node::{Builder, Config, Node, default_config};
use ldk_node::bitcoin::Network;

use std::thread;
use core::time::Duration;

pub fn run() -> () {
    let mut config = default_config();
    config.network = Network::Signet;
    let mut builder = Builder::from_config(config);
    builder.set_esplora_server("https://mutinynet.com/api/".to_string());
	builder.set_listening_addresses(vec![SocketAddress::TcpIpV4 {
		addr: [127, 0, 0, 1],
		port: 9376
	}]).unwrap();
	let node_a = Arc::new(builder.build().unwrap());

    node_a.start().unwrap();
    let event_node = Arc::clone(&node_a);
    std::thread::spawn(move || loop {
        let event = event_node.wait_next_event();
        println!("GOT NEW EVENT: {:?}", event);
        println!("Channels: {:?}", event_node.list_channels());
        println!("Payments: {:?}", event_node.list_payments());
        event_node.event_handled();
    });

    let mut config = default_config();
    config.network = Network::Signet;
    let mut builder = Builder::from_config(config);
    builder.set_esplora_server("https://mutinynet.com/api/".to_string());
	builder.set_listening_addresses(vec![SocketAddress::TcpIpV4 {
		addr: [127, 0, 0, 1],
		port: 9377
	}]).unwrap();
    let node_b = Arc::new(builder.build().unwrap());
    node_b.start().unwrap();
    
    node_a.connect_open_channel(
        node_b.node_id(),
        node_b.listening_addresses().unwrap().first().unwrap().clone(),
        10_000,
        None,
        None,
        true).unwrap();

	let offer = node_a.bolt12_payment().receive(10_000, "testing").unwrap();
	println!("Node offer: {}", offer);

    println!("Node ID: {}", node_a.node_id());
	println!("Node listening address: {:?}", node_a.listening_addresses());
    println!("Address: {}", node_a.onchain_payment().new_address().unwrap());
    println!("Funds: {:?}", node_a.list_balances());
    println!("Channels: {:?}", node_a.list_channels());

    // Make a payment to the offer
    
    loop {
        let payment_id = node_a.bolt12_payment().send(&offer, Some("TODO".to_string())).unwrap();
        println!("*** Payment id: {:?} ***", payment_id);

        thread::sleep(Duration::from_secs(10));
    }

    pause();
    node_a.stop().unwrap();
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
