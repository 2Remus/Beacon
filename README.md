# 🗼 Beacon
### A high-performance, container-native Minecraft orchestrator.

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/backend-Rust-orange.svg)](https://www.rust-lang.org/)
[![Vue 3](https://img.shields.io/badge/frontend-Vue%203-42b883.svg)](https://vuejs.org/)
[![Docker](https://img.shields.io/badge/platform-Docker-blue.svg)](https://www.docker.com/)

**Beacon** is a lightweight management suite designed to simplify the deployment and scaling of Minecraft servers. Built with a **Rust (Axum)** control plane and a **Vue 3** dashboard, Beacon provides a "blazingly fast" interface to spawn, monitor, and back up containerized game instances with near-zero host overhead.

---

## ✨ Features

* ⚡ **One-Click Deployment:** Spin up Vanilla, Paper, or Forge servers in seconds.
* 📈 **Real-time Monitoring:** Live CPU/RAM stats and console streaming via WebSockets.
* 📦 **Container-First:** Every server runs in an isolated Docker environment for maximum security.
* 💾 **Automated Backups:** Integrated snapshot system to keep your worlds safe.
* 🛠️ **Developer-Friendly API:** A fully documented REST API for custom integrations.
* 🔐 **Enterprise SSO:** Identity management powered by Keycloak.

---
I SHOULD REALLY PROVIDE DEVELOPMENT INFO 


### Contribution Workflow
1. Fork the Project.
2. Create your Feature Branch (git checkout -b feature/AmazingFeature).
3. Commit your Changes (git commit -m 'Add some AmazingFeature').
4. Push to the Branch (git push origin feature/AmazingFeature).
5. Open a Pull Request.

---

## 🛡️ Security Note

IMPORTANT: Beacon requires access to the Docker Socket (/var/run/docker.sock). In production environments, it is highly recommended to use a Docker Socket Proxy to limit the API calls Beacon can make to only necessary container management functions.

---

## 📄 License
Distributed under the MIT License. See LICENSE for more information.
