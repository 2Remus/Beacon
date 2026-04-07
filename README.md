[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/backend-Rust-orange.svg)](https://www.rust-lang.org/)
[![Vue 3](https://img.shields.io/badge/frontend-Vue%203-42b883.svg)](https://vuejs.org/)
[![Docker](https://img.shields.io/badge/platform-Docker-blue.svg)](https://www.docker.com/)

📡 Project Beacon
The Sovereignty-First Game Server Orchestrator

Project Beacon is a high-performance, open-source desktop orchestrator designed to put game hosting back into the hands of the players. By combining a Rust-based native core with a modern Vue.js interface, Beacon provides a "Liquid Glass" experience for managing Minecraft instances, secure tunnels, and system-level sidecar dependencies.

🚀 The Vision
Hosting a local server should be a core memory, not a technical headache. Many developers and gamers grew up struggling with port-forwarding, unstable VPNs, or volatile "free" hosting services that could delete their worlds at any moment to save resources.

Beacon solves this by turning your own hardware into a professional-grade hosting node. It manages the complexity of the JVM, networking, and storage, so you can focus on the game.

🛠️ The Architecture
Beacon follows a Native Hybrid Architecture. Unlike traditional Electron apps that rely solely on JavaScript, Beacon offloads all high-stakes operations to a compiled Rust core.

The Layers

The Core (Systems Layer): A native Rust engine built with napi-rs. It handles low-level process spawning (JVM), filesystem guards, symlinking, and memory allocation.

The Glue (FFI Bridge): A type-safe bridge that maps Rust structs to TypeScript interfaces, ensuring that data like PIDs and RAM usage are passed with zero-copy efficiency.

The Shell (Presentation Layer): A Vue 3 dashboard running in an Electron environment, designed for high-density information and real-time feedback.

The Sidecars: Managed sub-processes including cloudflared for secure tunneling and nginx for local traffic orchestration.

✨ Features
📦 Automated Provisioning: Dynamic server directory creation with automated eula.txt and server.properties generation.

⚙️ Lifecycle Management: Real-time PID tracking. Start, stop, and monitor JVM health directly from the dashboard.

🔒 Tunnel-Ready: Force-binds instances to loopback (127.0.0.1) by default, preparing them for secure public exposure via Cloudflare Sidecars.

💾 Registry System: A persistent JSON-based database that tracks every instance, its version, its provider (Vanilla/Paper/Fabric/Forge), and its resource allocation.

📂 Project Structure
  backendv2/ — The Heart. Native logic for server spawning and filesystem orchestration.

frontendv2/ — The Face. Vue 3 components, state management, and "Liquid Glass" styling.

(found after install )containers/ — The Vault. Where your isolated Minecraft instances live.

🚦 Getting Started
Prerequisites

Rust: 2021.

Node.js: v18 or higher.

Java: Version 17 or 21 (Must be available in your system PATH).

Installation

Clone the Repo:

git clone https://github.com/adafaralph/beacon.git

cd beacon

Install JS Dependencies:

yarn install

Compile the Native Core:

npm run build

Launch Beacon:

yarn dev

📜 Development Roadmap
[x] Native Rust-to-JS Bridge Implementation

[x] Dynamic Instance Provisioning

[x] JVM Process Lifecycle Management

[ ] Integrated Cloudflare Tunnel Configuration

[ ] Real-time Console Log Streaming via Rust Pipes

[ ] Resource Usage Graphs (CPU/RAM)

🤝 Contributing
Beacon is an open-source project. Whether you are a Rustacean, a Vue expert, or a Minecraft enthusiast, your contributions are welcome. Please feel free to open issues or submit pull requests.

⚖️ License
Distributed under the MIT License. See LICENSE for more information.

Crafted with 🦀 by Adafa Ralph
