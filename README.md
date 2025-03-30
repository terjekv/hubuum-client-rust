# Hubuum client library (Rust)

A Rust client library for interacting with the Hubuum API. The library is designed to be both flexible and safe, employing a type state pattern for authentication and offering both synchronous and asynchronous interfaces.

## Features

- **Type State Pattern for Authentication**:

    The client is built around a type state pattern. A new client instance is initially in an unauthenticated state (i.e. `Client<Unauthenticated>`) and only exposes the login interface. Once authenticated (via username/password or token), the client transitions to `Client<Authenticated>`, unlocking the full range of API operations.

- **Dual-Mode Operation**:

    Choose between a synchronous (blocking) or asynchronous (non-blocking) client depending on your application needs.
  
- **Robust Builder Interface**:

    A fluent builder pattern allows you to configure the client’s base URL, authentication details, timeout settings, and more before instantiation.

- **Comprehensive API Access**:

    Easily interact with resources such as classes, class relations, and other Hubuum API endpoints with well-defined method chains for filtering and execution.

## Installation

Add the dependency to your project's Cargo.toml (not yet available from `crates.io`):

```toml
[dependencies]
hubuum-client-rust = { git = "https://github.com/terjekv/hubuum-client-rust" }
```

## Usage

The library offers both a sync and an async client. The interface for both is similar, but the async client adds `await` syntax for asynchronous operations.

It is safe to `clone()` the client if need be.

### Synchronous Client

The synchronous client provides a blocking interface that is ideal for simpler or legacy applications.

#### Client Initialization and Authentication

```rust
use std::str::FromStr;
use hubuum_client::{BaseUrl, SyncClient, Token, Credentials};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let baseurl = BaseUrl::from_str(&format!("https://server.example.com:443"))?;

    // Create a new client in the Unauthenticated state
    let client = SyncClient::new(baseurl);

    // Log in using username; login returns a Client in the Authenticated state or an error.
    let client = client.login(Credentials::new("foo".to_string(), password.to_string()))?;
    // Alternatively, log in with a token:
    let client = client.login_with_token(Token::new(token.to_string()))?;

    Ok(())
}
```

#### Making API Calls

Once authenticated, you can perform operations against the API. For example, to create a new class resource:

```rust
use hubuum_client::models::ClassPost;

let result = client.classes().create(ClassPost {
    name: new.name.clone(),
    namespace_id: namespace.id,
    description: new.description.clone(),
    json_schema: new.json_schema.clone(),
    validate_schema: new.validate_schema,
})?;
```

Each endpoint has a corresponding method in the client, and each `POST` request is represented by a struct named `(type]POST` that implements the `Serialize` trait. The client handles the serialization and deserialization of these structs automatically.

#### Searching Resources

The client’s API is designed with a fluent query interface. For example, to search for a class by its exact name:

```rust
let class = client
    .classes()
    .find()
    .add_filter_name_exact(name)
    .execute_expecting_single_result()?;
```

Or, to find a relation between classes:

```rust
let class = client
        .class_relation()
        .find()
        .add_filter_equals("from_classes", from_class_id)
        .add_filter_equals("to_classes", to_class_id)
        .execute_expecting_single_result()?;
```

### Asynchronous Client

The asynchronous client leverages Rust’s async/await syntax and is built for high-concurrency applications using runtimes like Tokio.

#### Async Client Initialization and Authentication

```rust
use std::str::FromStr;
use hubuum_client::{BaseUrl, AsyncClient};

# [tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let baseurl = BaseUrl::from_str(&format!("https://server.example.com:443"))?;

    // Create a new asynchronous client in the Unauthenticated state
    let client = AsyncClient::new(baseurl);

    // Log in using username; login returns a Client in the Authenticated state or an error.
    let client = client.login(Credentials::new("foo".to_string(), password.to_string())).await?;
    // Alternatively, log in with a token:
    let client = client.login_with_token(Token::new(token.to_string())).await?;

    Ok(())
}
```

As one can see, the interface is very similar to the synchronous client.

## Contributing

Contributions are welcome! If you find issues or have suggestions for improvements, please open an issue or submit a pull request on GitHub.

## License

Distributed under the MIT License. See LICENSE for more details.
