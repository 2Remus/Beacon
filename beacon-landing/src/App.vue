<script setup>
// In a real scenario, these would point to your hosted artifacts or GitHub releases
const winDownload = "#";
const macDownload = "#";
</script>

<template>
  <div class="landing-container">
    <nav class="navbar">
      <div class="logo">BEACON</div>
      <div class="nav-links">
        <a href="#features">Features</a>
        <a href="https://github.com/2remus/Beacon" target="_blank">Source</a>
      </div>
    </nav>

    <main class="hero">
      <h1 class="title">The Intelligent <span class="accent">Server Orchestrator</span></h1>
      <p class="subtitle">
        Deploy, manage, and scale your Minecraft infrastructure with a high-performance Rust core
        and secure Cloudflare tunneling.
      </p>

      <div class="download-section">
        <div class="platform-card">
          <div class="icon">🪟</div>
          <h3>Windows</h3>
          <p>Requires Windows 10+</p>
          <a :href="winDownload" class="download-btn">Download .exe</a>
        </div>

        <div class="platform-card">
          <div class="icon">🍎</div>
          <h3>macOS</h3>
          <p>Apple Silicon & Intel</p>
          <a :href="macDownload" class="download-btn secondary">Download .dmg</a>
        </div>
      </div>
    </main>

    <footer class="footer">
      <p>&copy; 2026 Beacon. Built for performance.</p>
    </footer>
  </div>
</template>

<style scoped>
:global(body) {
  margin: 0;
  padding: 0;
  background-color: #0f172a;
  color: #f8fafc;
  font-family: 'Inter', system-ui, sans-serif;
}

.landing-container {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.navbar {
  display: flex;
  justify-content: space-between;
  padding: 2rem 10%;
  align-items: center;
}

.logo {
  font-weight: 800;
  letter-spacing: 2px;
  font-size: 1.5rem;
  color: #38bdf8;
}

.nav-links a {
  margin-left: 2rem;
  text-decoration: none;
  color: #94a3b8;
  transition: color 0.3s;
}

.nav-links a:hover {
  color: #f8fafc;
}

.hero {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 0 10%;
}

.title {
  font-size: 4rem;
  font-weight: 800;
  max-width: 800px;
  line-height: 1.1;
  margin-bottom: 1.5rem;
}

.accent {
  color: #38bdf8;
}

.subtitle {
  font-size: 1.25rem;
  color: #94a3b8;
  max-width: 600px;
  margin-bottom: 4rem;
}

.download-section {
  display: flex;
  gap: 2rem;
  flex-wrap: wrap;
  justify-content: center;
}

.platform-card {
  background: #1e293b;
  padding: 2.5rem;
  border-radius: 1rem;
  border: 1px solid #334155;
  width: 280px;
  transition: transform 0.3s, border-color 0.3s;
}

.platform-card:hover {
  transform: translateY(-5px);
  border-color: #38bdf8;
}

.icon {
  font-size: 3rem;
  margin-bottom: 1rem;
}

.download-btn {
  display: inline-block;
  margin-top: 1.5rem;
  padding: 0.75rem 1.5rem;
  background: #38bdf8;
  color: #0f172a;
  text-decoration: none;
  font-weight: 700;
  border-radius: 0.5rem;
  width: 80%;
  transition: opacity 0.3s;
}

.download-btn.secondary {
  background: #f8fafc;
}

.download-btn:hover {
  opacity: 0.9;
}

.footer {
  text-align: center;
  padding: 2rem;
  color: #475569;
  font-size: 0.875rem;
}

@media (max-width: 768px) {
  .title { font-size: 2.5rem; }
}
</style>