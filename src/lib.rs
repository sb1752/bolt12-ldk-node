use std::io;
// use std::io::prelude::*;
use ldk_node::lightning::ln::msgs::SocketAddress;
use std::sync::Arc;
use ldk_node::{Builder, Config, Node, default_config};
use ldk_node::bitcoin::Network;

use std::thread;
use core::time::Duration;

const DEBUG: bool = false;

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
        if DEBUG { println!("GOT NEW EVENT: {:?}", event); }
        if DEBUG { println!("Channels: {:?}", event_node.list_channels()); }
        if DEBUG { println!("Payments: {:?}", event_node.list_payments()); }
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
	if DEBUG { println!("Node offer: {}", offer); }

    if DEBUG { println!("Node ID: {}", node_a.node_id()); }
	if DEBUG { println!("Node listening address: {:?}", node_a.listening_addresses()); }
    if DEBUG { println!("Address: {}", node_a.onchain_payment().new_address().unwrap()); }
    if DEBUG { println!("Funds: {:?}", node_a.list_balances()); }
    if DEBUG { println!("Channels: {:?}", node_a.list_channels()); }

    loop {
        println!("> Are you ok this week? ");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim().to_lowercase();

        if input == "ok" {
            let payment_id = node_a.bolt12_payment().send(&offer, Some("TODO".to_string())).unwrap();
            if DEBUG { println!("*** Payment id: {:?} ***", payment_id); }
            println!("-- Thank you for your payment");
        } else if input == "exit" {
            node_a.stop().unwrap();
            node_b.stop().unwrap();
            break;
        } else {
            println!("-- Since you are sick you don't get to pay"); 
        }
        thread::sleep(Duration::from_secs(10));
    }
}
