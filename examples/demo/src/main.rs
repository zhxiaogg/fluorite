#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::wildcard_enum_match_arm)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::wildcard_enum_match_arm,
        deprecated
    )
)]

use std::fs;
use std::io::Write;
use std::path::Path;

use chrono::Utc;
use clap::{Parser, Subcommand};
use rust_decimal::Decimal;
use uuid::Uuid;

mod common {
    include!(concat!(env!("OUT_DIR"), "/common/mod.rs"));
}

mod demo {
    include!(concat!(env!("OUT_DIR"), "/demo/mod.rs"));
}

use common::{Address, Gender, Status};
use demo::{DemoEvent, MessagePayload, Order, OrderItem, User};

#[derive(Parser)]
#[command(name = "demo")]
#[command(about = "Fluorite demo with interop testing support")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Write sample data to JSON files
    Write {
        /// Output directory for JSON files
        #[arg(short, long, default_value = "./fixtures")]
        output: String,
    },
    /// Read and validate JSON files
    Read {
        /// Input directory containing JSON files
        #[arg(short, long, default_value = "./fixtures")]
        input: String,
    },
}

fn create_sample_address() -> Address {
    Address {
        street1: "123 Main St".to_string(),
        street2: Some("Apt 4B".to_string()),
        city: "Springfield".to_string(),
        state: "IL".to_string(),
        postal_code: "62701".to_string(),
        country: "US".to_string(),
    }
}

fn create_sample_user_male() -> User {
    User {
        id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap_or_default(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        age: 30,
        gender: Gender::Male,
        status: Status::Active,
        active: true,
        info: Some(serde_json::json!({
            "hobbies": ["reading", "coding"],
            "score": 95.5
        })),
        created_at: Utc::now(),
    }
}

fn create_sample_user_female() -> User {
    User {
        id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap_or_default(),
        first_name: "Jane".to_string(),
        last_name: "Smith".to_string(),
        age: 25,
        gender: Gender::Female,
        status: Status::Inactive,
        active: false,
        info: None,
        created_at: Utc::now(),
    }
}

fn create_sample_order() -> Order {
    Order {
        id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440003").unwrap_or_default(),
        user_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap_or_default(),
        items: vec![
            OrderItem {
                product_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440010")
                    .unwrap_or_default(),
                name: "Widget".to_string(),
                quantity: 2,
                unit_price: Decimal::new(1999, 2), // $19.99
            },
            OrderItem {
                product_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440011")
                    .unwrap_or_default(),
                name: "Gadget".to_string(),
                quantity: 1,
                unit_price: Decimal::new(4999, 2), // $49.99
            },
        ],
        total: Decimal::new(8997, 2), // $89.97
        shipping_address: create_sample_address(),
        created_at: Utc::now(),
        tracking_number: Some("1Z999AA10123456784".to_string()),
    }
}

fn create_event_user_created() -> DemoEvent {
    DemoEvent::UserCreated(create_sample_user_male())
}

fn create_event_order_placed() -> DemoEvent {
    DemoEvent::OrderPlaced(create_sample_order())
}

fn create_event_message() -> DemoEvent {
    DemoEvent::Message(MessagePayload {
        content: "Hello from Fluorite!".to_string(),
    })
}

fn create_event_ping() -> DemoEvent {
    DemoEvent::Ping
}

fn write_sample_data(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(output_dir);
    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    // Write User samples
    let user_male = create_sample_user_male();
    let user_female = create_sample_user_female();

    let mut file = fs::File::create(path.join("user_male.json"))?;
    file.write_all(serde_json::to_string_pretty(&user_male)?.as_bytes())?;

    let mut file = fs::File::create(path.join("user_female.json"))?;
    file.write_all(serde_json::to_string_pretty(&user_female)?.as_bytes())?;

    // Write Order sample
    let order = create_sample_order();
    let mut file = fs::File::create(path.join("order.json"))?;
    file.write_all(serde_json::to_string_pretty(&order)?.as_bytes())?;

    // Write DemoEvent samples (all variants)
    let events = [
        ("event_user_created.json", create_event_user_created()),
        ("event_order_placed.json", create_event_order_placed()),
        ("event_message.json", create_event_message()),
        ("event_ping.json", create_event_ping()),
    ];

    for (filename, event) in events {
        let mut file = fs::File::create(path.join(filename))?;
        file.write_all(serde_json::to_string_pretty(&event)?.as_bytes())?;
    }

    // Write Address sample
    let address = create_sample_address();
    let mut file = fs::File::create(path.join("address.json"))?;
    file.write_all(serde_json::to_string_pretty(&address)?.as_bytes())?;

    println!("Sample data written to {}", output_dir);
    Ok(())
}

fn read_and_validate(input_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(input_dir);

    // Validate Users
    for filename in ["user_male.json", "user_female.json"] {
        let file_path = path.join(filename);
        if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            let user: User = serde_json::from_str(&content)?;
            println!(
                "Validated User: {} {} ({})",
                user.first_name, user.last_name, user.id
            );
            println!(
                "  Gender: {:?}, Status: {:?}, Active: {}",
                user.gender, user.status, user.active
            );
        }
    }

    // Validate Order
    let order_path = path.join("order.json");
    if order_path.exists() {
        let content = fs::read_to_string(&order_path)?;
        let order: Order = serde_json::from_str(&content)?;
        println!(
            "Validated Order: {} with {} items, total: {}",
            order.id,
            order.items.len(),
            order.total
        );
        println!(
            "  Shipping to: {}, {}",
            order.shipping_address.city, order.shipping_address.country
        );
    }

    // Validate Events
    let event_files = [
        "event_user_created.json",
        "event_order_placed.json",
        "event_message.json",
        "event_ping.json",
    ];

    for filename in event_files {
        let file_path = path.join(filename);
        if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            let event: DemoEvent = serde_json::from_str(&content)?;
            match event {
                DemoEvent::UserCreated(user) => {
                    println!("Validated DemoEvent::UserCreated for {}", user.first_name);
                }
                DemoEvent::OrderPlaced(order) => {
                    println!("Validated DemoEvent::OrderPlaced for order {}", order.id);
                }
                DemoEvent::Message(msg) => {
                    println!("Validated DemoEvent::Message: {}", msg.content);
                }
                DemoEvent::Ping => {
                    println!("Validated DemoEvent::Ping");
                }
            }
        }
    }

    // Validate Address
    let address_path = path.join("address.json");
    if address_path.exists() {
        let content = fs::read_to_string(&address_path)?;
        let address: Address = serde_json::from_str(&content)?;
        println!("Validated Address: {}, {}", address.city, address.country);
    }

    Ok(())
}

fn run_default_demo() {
    let user = create_sample_user_male();
    println!("User: {:?}", user);

    let order = create_sample_order();
    println!("Order: {:?}", order);

    let event = create_event_message();
    println!("Event: {:?}", event);
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Write { output }) => {
            if let Err(e) = write_sample_data(&output) {
                eprintln!("Error writing sample data: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Read { input }) => {
            if let Err(e) = read_and_validate(&input) {
                eprintln!("Error reading sample data: {}", e);
                std::process::exit(1);
            }
        }
        None => {
            run_default_demo();
        }
    }
}
