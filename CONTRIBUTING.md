# 🛠 Contributing to Beacon Hub

Thank you for your interest in contributing! This guide will help you get your local development environment synced with the Project Beacon architecture.

---

## 💻 Development Environment Setup

### 1. Repository Structure
The project is a monorepo containing several distinct environments. Ensure your IDE (VS Code, Zed, or IntelliJ) is opened at the **root** of the repository.

- /backend: Rust (Axum/Tokio) orchestrator.
- /frontend: Vue 3 + Vite + Tailwind CSS.
- /beacon-host: Rust-based CLI for node management.
- /database: SQL schemas and migration scripts.

### 2. Required Toolchain
Install the following dependencies based on your operating system:

- Rust: rustup (Edition 2021).
- Node.js: v20+ (with npm).
- Docker Desktop: Must be running for container orchestration tests.
- libpq: Required for local database connectivity (if running tests outside Docker).
  - Mac: brew install libpq
  - Linux: sudo apt install libpq-dev

### 3. Setting Up the Backend (Rust)
If you are working on the orchestrator logic:
1. Copy the environment template: cp .env.example .env.
2. Ensure Docker is running.
3. Use cargo to run the orchestrator in dev mode:
   cd backend
   cargo run

### 4. Setting Up the Frontend (Vue/Vite)
To modify the dashboard:
1. Navigate to the frontend directory: cd frontend.
2. Install dependencies: npm install.
3. Start the development server: npm run dev.
4. The dashboard will be available at http://localhost:5173.

---

## 🐳 Docker Workflow
Beacon uses Docker to manage the control plane. 

- Rebuilding Images: If you modify the backend binary and want to test it in the container, ensure you compile for the correct architecture.
- Apple Silicon: Use platform: linux/amd64 in docker-compose.yml if running x86_64 binaries via Rosetta.
- Database Reset: Use the provided scripts in the root directory:
  - ./setup_db.sh (Mac/Linux)
  - setup_db.bat (Windows)

---

## 🧪 Testing Guidelines
- Unit Tests: Run cargo test in the /backend directory.
- Integration: Verify that the orchestrator can correctly trigger docker compose up by running the binary as an Administrator/Sudo (required for hosts file modification).

---

## 📜 Pull Request Process
1. Create a new branch: git checkout -b dev/your-feature-name.
2. Ensure your code follows the existing style (run cargo fmt and npm run lint).
3. Commit with descriptive messages.
4. Push to your fork and submit a PR to the main branch.
