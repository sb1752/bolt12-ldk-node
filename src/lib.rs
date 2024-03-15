use std::io;
// use std::io::prelude::*;
use ldk_node::lightning::ln::msgs::SocketAddress;
use std::sync::Arc;
use ldk_node::{Builder, Config, Node, default_config};
use ldk_node::bitcoin::Network;

use std::thread;
use core::time::Duration;
use ldk_node::LogLevel;

const DEBUG: bool = true;

pub fn run() -> () {
    let mut config_a = default_config();
    config_a.network = Network::Signet;
    config_a.storage_dir_path = "/tmp/ldk_node_a".to_string();
    config_a.log_level = LogLevel::Trace;
    let mut builder_a = Builder::from_config(config_a);
    builder_a.set_esplora_server("https://mutinynet.com/api/".to_string());
	builder_a.set_listening_addresses(vec![SocketAddress::TcpIpV4 {
		addr: [127, 0, 0, 1],
		port: 9376
	}]).unwrap();
	let node_a = Arc::new(builder_a.build().unwrap());
    node_a.start().unwrap();

    let mut config_b = default_config();
    config_b.network = Network::Signet;
    config_b.log_level = LogLevel::Trace;
    config_b.storage_dir_path = "/tmp/ldk_node_b".to_string();
    let mut builder_b = Builder::from_config(config_b);
    builder_b.set_esplora_server("https://mutinynet.com/api/".to_string());
	builder_b.set_listening_addresses(vec![SocketAddress::TcpIpV4 {
		addr: [127, 0, 0, 1],
		port: 9377
	}]).unwrap();
    let node_b = Arc::new(builder_b.build().unwrap());
    node_b.start().unwrap();

    println!("node a id: {:?}", node_a.node_id());
    println!("node b id: {:?}", node_b.node_id());
    if DEBUG { println!("Node A Address: {}", node_a.onchain_payment().new_address().unwrap()); }
    if DEBUG { println!("Node B Address: {}", node_b.onchain_payment().new_address().unwrap()); }

    let event_node = Arc::clone(&node_a);
    std::thread::spawn(move || loop {
        let event = event_node.wait_next_event();
        if DEBUG { println!("NODE A: {:?}", event); }
        event_node.event_handled();
    });

    let event_node = Arc::clone(&node_b);
    std::thread::spawn(move || loop {
        let event = event_node.wait_next_event();
        if DEBUG { println!("NODE B: {:?}", event); }
        event_node.event_handled();
    });

    // if DEBUG { println!("Node ID: {}", node_a.node_id()); }
	// if DEBUG { println!("Node listening address: {:?}", node_a.listening_addresses()); }
    // if DEBUG { println!("Funds: {:?}", node_a.list_balances()); }

    let channel_id = node_a.connect_open_channel(
        node_b.node_id(),
        node_b.listening_addresses().unwrap().first().unwrap().clone(),
        10_000,
        None,
        None,
        false).unwrap();

    if DEBUG { println!("Channels: {:?}", node_a.list_channels()); }
    if DEBUG { println!("Channel ID: {:?}", channel_id)};
    
    let mut offer = node_b.bolt12_payment().receive(10_000, "testing").unwrap();
    if DEBUG { println!("Node offer: {}", offer); }

    loop {
        println!("> Are you ok this week? ");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim().to_lowercase();

        if input == "ok" {
            let payment_id = node_a.bolt12_payment().send(&offer, Some("TODO".to_string())).unwrap();
            println!("Thank you for your payment");
            println!("Payment ID: {:?}", payment_id);
            println!("Payment Status: {:?}", node_a.list_payments_with_filter(|p| p.id == payment_id).first().unwrap().status);
            println!("Node A balance: {:?}", node_a.list_balances().total_lightning_balance_sats);
        } else if input == "exit" {
            node_a.stop().unwrap();
            node_b.stop().unwrap();
            break;
        } else if input == "create offer" {
            offer = node_b.bolt12_payment().receive(10_000, "testing").unwrap();
            if DEBUG { println!("Node offer: {}", offer); }
        } else {
            println!("-- Since you are sick you don't get to pay"); 
            println!("Node A balance: {:?}", node_a.list_balances().total_lightning_balance_sats);
        }
        thread::sleep(Duration::from_secs(5));
    }
}
