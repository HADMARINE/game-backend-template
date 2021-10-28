# Game-backend-template

## Prerequisites on your environment

- NPM / Node.js [Link](https://nodejs.org/en/)
- Rust (cargo) [Link](https://www.rust-lang.org/tools/install)
- YARN [Link](https://yarnpkg.com/)

## Tech stacks

- Node.js
  - Express
  - MongoDB
  - [Express-Quick-Builder](https://github.com/hadmarine/express-quick-builder)
- Rust
  - Tungstenite
  - wasm-bindgen
  <!-- - Docker -->

## What action can be done with this template

- HTTP Connection with Express
- Multi-threaded TCP(WebSocket)/UDP transmission implemented with Rust
- Event-Based transmission between Rust instance and Node.JS instance

## Known issues

- Cannot stablize PnP such as YARN 2.0 because of Rust compilation
