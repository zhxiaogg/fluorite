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
