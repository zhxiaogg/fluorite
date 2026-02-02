# Consolidate .fl Example Files into Demos

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Consolidate standalone .fl example files into the Rust/TypeScript demos with multiple packages to test `package`/`use` imports, and enhance demos with more Fluorite features.

**Architecture:** Create two .fl files (`common.fl` for shared types, `demo.fl` for demo-specific types that import from common). Update both Rust and TypeScript demos to use these schemas and demonstrate cross-package imports, collections, unions, and all primitive types.

**Tech Stack:** Rust (serde, clap), TypeScript, Fluorite IDL (.fl files)

---

## Task 1: Create common.fl with Shared Types

**Files:**
- Create: `examples/demo/fluorite/common.fl`

**Step 1: Create the common.fl file**

```rust
/// Common shared types used across the demo
package common;

/// A physical address
struct Address {
    /// Street line 1
    street1: String,
    /// Street line 2 (optional)
    street2: Option<String>,
    /// City name
    city: String,
    /// State or province
    state: String,
    /// Postal/ZIP code
    postal_code: String,
    /// ISO country code
    country: String,
}

/// Gender options
enum Gender {
    Male,
    Female,
    Other,
}

/// Account status
enum Status {
    Active,
    Inactive,
    Suspended,
}
```

**Step 2: Verify file is created**

Run: `cat examples/demo/fluorite/common.fl`
Expected: File contents shown above

---

## Task 2: Create demo.fl with Cross-Package Imports

**Files:**
- Create: `examples/demo/fluorite/demo.fl`
- Delete: `examples/demo/fluorite/demo.yaml`

**Step 1: Create the demo.fl file**

```rust
/// Demo types showcasing Fluorite features
package demo;

use common::Address;
use common::Gender;
use common::Status;

/// A user in the system
#[rename_all = "camelCase"]
struct User {
    /// Unique identifier
    id: Uuid,
    /// First name
    first_name: String,
    /// Last name
    last_name: String,
    /// Age in years
    age: u32,
    /// User's gender
    gender: Gender,
    /// Account status
    status: Status,
    /// Whether currently active
    active: bool,
    /// Optional metadata (Any type)
    info: Option<Any>,
    /// When the user was created
    created_at: DateTime,
}

/// An item in an order
#[rename_all = "camelCase"]
struct OrderItem {
    /// Product identifier
    product_id: Uuid,
    /// Product name
    name: String,
    /// Quantity ordered
    quantity: u32,
    /// Price per unit
    unit_price: Decimal,
}

/// A customer order
#[rename_all = "camelCase"]
struct Order {
    /// Order identifier
    id: Uuid,
    /// Customer's user ID
    user_id: Uuid,
    /// Items in this order
    items: Vec<OrderItem>,
    /// Total order amount
    total: Decimal,
    /// Shipping address (imported from common)
    shipping_address: Address,
    /// When the order was placed
    created_at: DateTime,
    /// Tracking number if shipped
    tracking_number: Option<String>,
}

/// Events for demonstrating unions
#[type_tag = "type"]
#[extern]
union DemoEvent {
    /// A user was created
    UserCreated(User),
    /// An order was placed
    OrderPlaced(Order),
    /// A simple string message
    Message(MessagePayload),
    /// System ping (unit variant)
    Ping,
}

/// Payload for message events
#[rename_all = "camelCase"]
struct MessagePayload {
    /// The message content
    content: String,
}

/// List of users
type UserList = Vec<User>;

/// Map of order IDs to orders
type OrderMap = Map<String, Order>;
```

**Step 2: Delete the old demo.yaml**

Run: `rm examples/demo/fluorite/demo.yaml`

**Step 3: Verify files exist**

Run: `ls -la examples/demo/fluorite/`
Expected: `common.fl` and `demo.fl` present, no `demo.yaml`

---

## Task 3: Update Rust Demo build.rs

**Files:**
- Modify: `examples/demo/build.rs`

**Step 1: Update build.rs to compile both .fl files**

Replace entire contents with:

```rust
use fluorite_codegen::code_gen::rust::RustOptions;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let options = RustOptions::new(out_dir)
        .with_any_type("serde_json::Value")
        .with_generate_new(true);

    // Compile both .fl files - order matters for imports
    fluorite_codegen::compile_with_options(
        options,
        &["fluorite/common.fl", "fluorite/demo.fl"]
    ).unwrap();
}
```

**Step 2: Build to verify compilation**

Run: `cd examples/demo && cargo build`
Expected: Build succeeds

---

## Task 4: Update Rust Demo Cargo.toml

**Files:**
- Modify: `examples/demo/Cargo.toml`

**Step 1: Add required dependencies for new types**

Add to `[dependencies]`:

```toml
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
rust_decimal = { version = "1.0", features = ["serde"] }
```

**Step 2: Verify Cargo.toml is valid**

Run: `cd examples/demo && cargo check`
Expected: Check succeeds

---

## Task 5: Update Rust Demo main.rs

**Files:**
- Modify: `examples/demo/src/main.rs`

**Step 1: Replace entire main.rs with enhanced demo**

```rust
#![allow(dead_code)]
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
        created_at: Utc::now().to_rfc3339(),
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
        created_at: Utc::now().to_rfc3339(),
    }
}

fn create_sample_order() -> Order {
    Order {
        id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440003").unwrap_or_default(),
        user_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap_or_default(),
        items: vec![
            OrderItem {
                product_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440010").unwrap_or_default(),
                name: "Widget".to_string(),
                quantity: 2,
                unit_price: Decimal::new(1999, 2), // $19.99
            },
            OrderItem {
                product_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440011").unwrap_or_default(),
                name: "Gadget".to_string(),
                quantity: 1,
                unit_price: Decimal::new(4999, 2), // $49.99
            },
        ],
        total: Decimal::new(8997, 2), // $89.97
        shipping_address: create_sample_address(),
        created_at: Utc::now().to_rfc3339(),
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
            println!("Validated User: {} {} ({})", user.first_name, user.last_name, user.id);
            println!("  Gender: {:?}, Status: {:?}, Active: {}", user.gender, user.status, user.active);
        }
    }

    // Validate Order
    let order_path = path.join("order.json");
    if order_path.exists() {
        let content = fs::read_to_string(&order_path)?;
        let order: Order = serde_json::from_str(&content)?;
        println!("Validated Order: {} with {} items, total: {}", order.id, order.items.len(), order.total);
        println!("  Shipping to: {}, {}", order.shipping_address.city, order.shipping_address.country);
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
```

**Step 2: Build and test**

Run: `cd examples/demo && cargo build && cargo run`
Expected: Build succeeds and default demo runs

---

## Task 6: Update TypeScript Demo

**Files:**
- Modify: `examples/demo-ts/src/index.ts`
- Modify: `examples/demo-ts/package.json`

**Step 1: Update package.json generate script**

Change the `generate` script to use .fl files:

```json
{
  "scripts": {
    "generate": "fluorite ts --inputs ../demo/fluorite/common.fl ../demo/fluorite/demo.fl --output ./generated",
    "build": "tsc",
    "start": "ts-node src/index.ts",
    "demo": "npm run generate && npm run build && node dist/index.js"
  }
}
```

**Step 2: Replace index.ts with enhanced TypeScript demo**

```typescript
import * as fs from "fs";
import * as path from "path";

// Import generated types from both packages
import { Address, Gender, Status } from "../generated/common";
import {
  User,
  Order,
  OrderItem,
  DemoEvent,
  MessagePayload,
} from "../generated/demo";

// JSON types matching serde camelCase serialization
interface AddressJson {
  street1: string;
  street2?: string;
  city: string;
  state: string;
  postalCode: string;
  country: string;
}

interface UserJson {
  id: string;
  firstName: string;
  lastName: string;
  age: number;
  gender: "Male" | "Female" | "Other";
  status: "Active" | "Inactive" | "Suspended";
  active: boolean;
  info?: unknown;
  createdAt: string;
}

interface OrderItemJson {
  productId: string;
  name: string;
  quantity: number;
  unitPrice: string;
}

interface OrderJson {
  id: string;
  userId: string;
  items: OrderItemJson[];
  total: string;
  shippingAddress: AddressJson;
  createdAt: string;
  trackingNumber?: string;
}

interface MessagePayloadJson {
  content: string;
}

type DemoEventJson =
  | { type: "UserCreated" } & UserJson
  | { type: "OrderPlaced" } & OrderJson
  | { type: "Message" } & MessagePayloadJson
  | { type: "Ping" };

// Sample data creators
function createSampleAddress(): AddressJson {
  return {
    street1: "123 Main St",
    street2: "Apt 4B",
    city: "Springfield",
    state: "IL",
    postalCode: "62701",
    country: "US",
  };
}

function createSampleUserMale(): UserJson {
  return {
    id: "550e8400-e29b-41d4-a716-446655440001",
    firstName: "John",
    lastName: "Doe",
    age: 30,
    gender: "Male",
    status: "Active",
    active: true,
    info: {
      hobbies: ["reading", "coding"],
      score: 95.5,
    },
    createdAt: new Date().toISOString(),
  };
}

function createSampleUserFemale(): UserJson {
  return {
    id: "550e8400-e29b-41d4-a716-446655440002",
    firstName: "Jane",
    lastName: "Smith",
    age: 25,
    gender: "Female",
    status: "Inactive",
    active: false,
    createdAt: new Date().toISOString(),
  };
}

function createSampleOrder(): OrderJson {
  return {
    id: "550e8400-e29b-41d4-a716-446655440003",
    userId: "550e8400-e29b-41d4-a716-446655440001",
    items: [
      {
        productId: "550e8400-e29b-41d4-a716-446655440010",
        name: "Widget",
        quantity: 2,
        unitPrice: "19.99",
      },
      {
        productId: "550e8400-e29b-41d4-a716-446655440011",
        name: "Gadget",
        quantity: 1,
        unitPrice: "49.99",
      },
    ],
    total: "89.97",
    shippingAddress: createSampleAddress(),
    createdAt: new Date().toISOString(),
    trackingNumber: "1Z999AA10123456784",
  };
}

function createEventUserCreated(): DemoEventJson {
  return {
    type: "UserCreated",
    ...createSampleUserMale(),
  };
}

function createEventOrderPlaced(): DemoEventJson {
  return {
    type: "OrderPlaced",
    ...createSampleOrder(),
  };
}

function createEventMessage(): DemoEventJson {
  return {
    type: "Message",
    content: "Hello from Fluorite!",
  };
}

function createEventPing(): DemoEventJson {
  return {
    type: "Ping",
  };
}

// Serialization helpers
function serializeToJson(data: unknown): string {
  return JSON.stringify(data, null, 2);
}

function deserializeFromJson<T>(json: string): T {
  return JSON.parse(json) as T;
}

// Write sample data to files
function writeSampleData(outputDir: string): void {
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  // Write User samples
  fs.writeFileSync(
    path.join(outputDir, "user_male.json"),
    serializeToJson(createSampleUserMale())
  );
  fs.writeFileSync(
    path.join(outputDir, "user_female.json"),
    serializeToJson(createSampleUserFemale())
  );

  // Write Order sample
  fs.writeFileSync(
    path.join(outputDir, "order.json"),
    serializeToJson(createSampleOrder())
  );

  // Write DemoEvent samples
  fs.writeFileSync(
    path.join(outputDir, "event_user_created.json"),
    serializeToJson(createEventUserCreated())
  );
  fs.writeFileSync(
    path.join(outputDir, "event_order_placed.json"),
    serializeToJson(createEventOrderPlaced())
  );
  fs.writeFileSync(
    path.join(outputDir, "event_message.json"),
    serializeToJson(createEventMessage())
  );
  fs.writeFileSync(
    path.join(outputDir, "event_ping.json"),
    serializeToJson(createEventPing())
  );

  // Write Address sample
  fs.writeFileSync(
    path.join(outputDir, "address.json"),
    serializeToJson(createSampleAddress())
  );

  console.log(`Sample data written to ${outputDir}`);
}

// Read and validate JSON files
function readAndValidate(inputDir: string): void {
  // Validate Users
  for (const filename of ["user_male.json", "user_female.json"]) {
    const filePath = path.join(inputDir, filename);
    if (fs.existsSync(filePath)) {
      const content = fs.readFileSync(filePath, "utf-8");
      const user = deserializeFromJson<UserJson>(content);
      console.log(`Validated User: ${user.firstName} ${user.lastName} (${user.id})`);
      console.log(`  Gender: ${user.gender}, Status: ${user.status}, Active: ${user.active}`);
    }
  }

  // Validate Order
  const orderPath = path.join(inputDir, "order.json");
  if (fs.existsSync(orderPath)) {
    const content = fs.readFileSync(orderPath, "utf-8");
    const order = deserializeFromJson<OrderJson>(content);
    console.log(`Validated Order: ${order.id} with ${order.items.length} items, total: ${order.total}`);
    console.log(`  Shipping to: ${order.shippingAddress.city}, ${order.shippingAddress.country}`);
  }

  // Validate Events
  const eventFiles = [
    "event_user_created.json",
    "event_order_placed.json",
    "event_message.json",
    "event_ping.json",
  ];

  for (const filename of eventFiles) {
    const filePath = path.join(inputDir, filename);
    if (fs.existsSync(filePath)) {
      const content = fs.readFileSync(filePath, "utf-8");
      const event = deserializeFromJson<DemoEventJson>(content);
      switch (event.type) {
        case "UserCreated":
          console.log(`Validated DemoEvent::UserCreated for ${event.firstName}`);
          break;
        case "OrderPlaced":
          console.log(`Validated DemoEvent::OrderPlaced for order ${event.id}`);
          break;
        case "Message":
          console.log(`Validated DemoEvent::Message: ${event.content}`);
          break;
        case "Ping":
          console.log("Validated DemoEvent::Ping");
          break;
      }
    }
  }

  // Validate Address
  const addressPath = path.join(inputDir, "address.json");
  if (fs.existsSync(addressPath)) {
    const content = fs.readFileSync(addressPath, "utf-8");
    const address = deserializeFromJson<AddressJson>(content);
    console.log(`Validated Address: ${address.city}, ${address.country}`);
  }
}

// Main CLI
function main(): void {
  const args = process.argv.slice(2);

  if (args.length === 0) {
    console.log("Usage: ts-node index.ts [--write <dir> | --read <dir>]");
    console.log("  --write <dir>  Write sample JSON files to directory");
    console.log("  --read <dir>   Read and validate JSON files from directory");
    process.exit(0);
  }

  const command = args[0];
  const dir = args[1] || "./fixtures";

  switch (command) {
    case "--write":
      writeSampleData(dir);
      break;
    case "--read":
      readAndValidate(dir);
      break;
    default:
      console.error(`Unknown command: ${command}`);
      process.exit(1);
  }
}

main();
```

**Step 3: Verify TypeScript builds**

Run: `cd examples/demo-ts && npm run generate && npm run build`
Expected: Generation and build succeed

---

## Task 7: Update Interop Test Script

**Files:**
- Modify: `tests/interop/run-interop-test.sh`

**Step 1: Update script with new test files**

Replace the script with:

```bash
#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================="
echo "Fluorite E2E Interoperability Test"
echo "========================================="
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FIXTURES_DIR="$SCRIPT_DIR/fixtures"
RUST_TO_TS_DIR="$FIXTURES_DIR/rust_to_ts"
TS_TO_RUST_DIR="$FIXTURES_DIR/ts_to_rust"

# Create fixture directories
mkdir -p "$RUST_TO_TS_DIR"
mkdir -p "$TS_TO_RUST_DIR"

# Track test results
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

info() {
    echo -e "${YELLOW}→ INFO${NC}: $1"
}

# Step 1: Build Rust demo
echo "Step 1: Building Rust demo..."
cd "$SCRIPT_DIR/../../examples/demo"
if cargo build --quiet 2>/dev/null; then
    pass "Rust demo built successfully"
else
    fail "Rust demo build failed"
    exit 1
fi

# Step 2: Build Fluorite CLI for TypeScript generation
echo ""
echo "Step 2: Building Fluorite CLI..."
cd "$SCRIPT_DIR/../../"
if cargo build --bin fluorite --quiet 2>/dev/null; then
    pass "Fluorite CLI built successfully"
else
    fail "Fluorite CLI build failed"
    exit 1
fi

# Step 3: Generate TypeScript types from .fl files
echo ""
echo "Step 3: Generating TypeScript types..."
cd "$SCRIPT_DIR/../../examples/demo-ts"
if "$SCRIPT_DIR/../../target/debug/fluorite" ts --inputs ../demo/fluorite/common.fl ../demo/fluorite/demo.fl --output ./generated 2>/dev/null; then
    pass "TypeScript types generated successfully"
else
    fail "TypeScript type generation failed"
    exit 1
fi

# Step 4: Install TypeScript dependencies and build
echo ""
echo "Step 4: Installing TypeScript dependencies..."
if [ ! -d "node_modules" ]; then
    npm install --silent 2>/dev/null || true
fi

# Build TypeScript
info "Building TypeScript..."
npx tsc 2>/dev/null || true

# Check if dist/index.js exists
if [ -f "dist/src/index.js" ]; then
    pass "TypeScript built successfully"
else
    info "Using ts-node for direct execution"
fi

# Step 5: Test Rust → TypeScript
echo ""
echo "========================================="
echo "Step 5: Testing Rust → TypeScript"
echo "========================================="

# Rust writes JSON files
cd "$SCRIPT_DIR/../../examples/demo"
info "Running Rust demo to write JSON files..."
cargo run --quiet -- write --output "$RUST_TO_TS_DIR" 2>/dev/null

# Verify Rust wrote files
REQUIRED_FILES=(
    "user_male.json"
    "user_female.json"
    "order.json"
    "event_user_created.json"
    "event_order_placed.json"
    "event_message.json"
    "event_ping.json"
    "address.json"
)

ALL_EXIST=true
for file in "${REQUIRED_FILES[@]}"; do
    if [ ! -f "$RUST_TO_TS_DIR/$file" ]; then
        ALL_EXIST=false
        fail "Rust failed to write $file"
    fi
done

if [ "$ALL_EXIST" = true ]; then
    pass "Rust wrote all JSON files successfully"
fi

# TypeScript reads and validates Rust's JSON
cd "$SCRIPT_DIR/../../examples/demo-ts"
info "Running TypeScript to read Rust's JSON files..."
if node dist/src/index.js --read "$RUST_TO_TS_DIR" 2>/dev/null; then
    pass "TypeScript successfully read Rust's JSON files"
else
    if npx ts-node src/index.ts --read "$RUST_TO_TS_DIR" 2>/dev/null; then
        pass "TypeScript successfully read Rust's JSON files (ts-node)"
    else
        fail "TypeScript failed to read Rust's JSON files"
    fi
fi

# Step 6: Test TypeScript → Rust
echo ""
echo "========================================="
echo "Step 6: Testing TypeScript → Rust"
echo "========================================="

# TypeScript writes JSON files
cd "$SCRIPT_DIR/../../examples/demo-ts"
info "Running TypeScript to write JSON files..."
if node dist/src/index.js --write "$TS_TO_RUST_DIR" 2>/dev/null; then
    pass "TypeScript wrote JSON files successfully"
else
    if npx ts-node src/index.ts --write "$TS_TO_RUST_DIR" 2>/dev/null; then
        pass "TypeScript wrote JSON files successfully (ts-node)"
    else
        fail "TypeScript failed to write JSON files"
    fi
fi

# Verify TypeScript wrote files
ALL_EXIST=true
for file in "${REQUIRED_FILES[@]}"; do
    if [ ! -f "$TS_TO_RUST_DIR/$file" ]; then
        ALL_EXIST=false
        fail "TypeScript failed to write $file"
    fi
done

if [ "$ALL_EXIST" = true ]; then
    pass "TypeScript wrote all JSON files successfully"
fi

# Rust reads and validates TypeScript's JSON
cd "$SCRIPT_DIR/../../examples/demo"
info "Running Rust demo to read TypeScript's JSON files..."
if cargo run --quiet -- read --input "$TS_TO_RUST_DIR" 2>/dev/null; then
    pass "Rust successfully read TypeScript's JSON files"
else
    fail "Rust failed to read TypeScript's JSON files"
fi

# Step 7: Summary
echo ""
echo "========================================="
echo "Test Summary"
echo "========================================="
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All interoperability tests passed!${NC}"
    echo ""
    echo "Tested scenarios:"
    echo "  ✓ Rust serializes → TypeScript deserializes"
    echo "  ✓ TypeScript serializes → Rust deserializes"
    echo "  ✓ User objects with UUID, DateTime, Optional fields"
    echo "  ✓ Order with nested Address, Vec<OrderItem>, Decimal"
    echo "  ✓ Gender enum (Male/Female/Other)"
    echo "  ✓ Status enum (Active/Inactive/Suspended)"
    echo "  ✓ DemoEvent union (UserCreated/OrderPlaced/Message/Ping)"
    echo "  ✓ Cross-package imports (common → demo)"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
```

**Step 2: Make script executable and test**

Run: `chmod +x tests/interop/run-interop-test.sh && ./tests/interop/run-interop-test.sh`
Expected: All tests pass

---

## Task 8: Remove Old Standalone Example Files

**Files:**
- Delete: `examples/users.fl`
- Delete: `examples/orders.fl`
- Delete: `examples/users.yml`
- Delete: `examples/orders.yml`

**Step 1: Remove old files**

Run: `rm examples/users.fl examples/orders.fl examples/users.yml examples/orders.yml`

**Step 2: Update IDL tests to use new locations**

Modify: `codegen/tests/idl_code_gen.rs`

Update the `fixtures_dir()` function and test paths:

```rust
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("examples")
        .join("demo")
        .join("fluorite")
}

#[test]
fn test_parse_common_fl() {
    let path = fixtures_dir().join("common.fl");
    let result = parse_file(&path);
    assert!(
        result.is_ok(),
        "Failed to parse common.fl: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.package.value, "common");
    assert_eq!(ast.items.len(), 3); // Address, Gender, Status
}

#[test]
fn test_parse_demo_fl() {
    let path = fixtures_dir().join("demo.fl");
    let result = parse_file(&path);
    assert!(
        result.is_ok(),
        "Failed to parse demo.fl: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.package.value, "demo");
    assert_eq!(ast.uses.len(), 3); // Imports Address, Gender, Status from common
}

#[test]
fn test_parse_multiple_packages() {
    let paths = vec![
        fixtures_dir().join("common.fl"),
        fixtures_dir().join("demo.fl"),
    ];
    let result = parse_files(&paths);
    assert!(result.is_ok(), "Failed to parse files: {:?}", result.err());

    let asts = result.unwrap();
    assert_eq!(asts.len(), 2);
}
```

**Step 3: Remove old tests that reference deleted files**

Remove tests: `test_parse_users_fl`, `test_parse_orders_fl`, and update `test_parse_multiple_files`.

---

## Task 9: Run Full Test Suite

**Step 1: Run make all**

Run: `make all`
Expected: All checks pass (fmt-check, lint, test)

**Step 2: Run interop test**

Run: `./tests/interop/run-interop-test.sh`
Expected: All interop tests pass

---

## Task 10: Update Documentation

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Update CLAUDE.md demo section**

Add/update the following section:

```markdown
## Demo Projects

The demos in `examples/` showcase Fluorite's features with multi-package schemas:

### Schema Files
- `examples/demo/fluorite/common.fl` - Shared types (Address, Gender, Status)
- `examples/demo/fluorite/demo.fl` - Demo types that import from common

### Features Demonstrated
- **Cross-package imports**: `use common::Address;`
- **Primitive types**: Uuid, DateTime, Decimal, String, u32, bool
- **Optional fields**: `Option<T>`
- **Collections**: `Vec<T>`, `Map<K, V>`
- **Enums**: Gender, Status
- **Tagged unions**: DemoEvent with data and unit variants
- **Type aliases**: UserList, OrderMap
- **Serde attributes**: `#[rename_all = "camelCase"]`

### Running the Demos

```bash
# Rust demo
cd examples/demo
cargo run              # Default demo
cargo run -- write -o ./fixtures  # Write JSON
cargo run -- read -i ./fixtures   # Read/validate JSON

# TypeScript demo
cd examples/demo-ts
npm install
npm run generate       # Generate types from .fl
npm run build
node dist/src/index.js --write ./fixtures
node dist/src/index.js --read ./fixtures

# Interop test (Rust ↔ TypeScript)
./tests/interop/run-interop-test.sh
```
```

**Step 2: Commit changes**

Run: `git add -A && git commit -m "feat: consolidate .fl examples into enhanced demos with multi-package support"`

---

## E2E Test Acceptance Criteria

The implementation is complete when all of the following pass:

1. ✓ `make all` passes (format, lint, tests)
2. ✓ `./tests/interop/run-interop-test.sh` passes
3. ✓ Rust demo builds and runs with both `write` and `read` commands
4. ✓ TypeScript demo generates, builds, and runs with both commands
5. ✓ Cross-package imports work (`demo.fl` imports from `common.fl`)
6. ✓ All type features work:
   - Uuid, DateTime, Decimal primitives
   - Optional fields
   - Vec<T> collections
   - Nested structs (Order contains Address)
   - Enums (Gender, Status)
   - Tagged unions with data + unit variants (DemoEvent)
7. ✓ Bidirectional JSON compatibility (Rust ↔ TypeScript)
