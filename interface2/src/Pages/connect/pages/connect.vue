<script setup>
import { ref, computed, onUnmounted } from 'vue'

// --- State Management ---
const tunnelUrl = ref('') // Restored
const connectionStatus = ref('idle')
const remoteInstances = ref([])
const isScanning = ref(false)

// --- Computed UI State ---
const isConnected = computed(() => connectionStatus.value === 'active')
const statusLabel = computed(() => ({
  idle: 'OFFLINE',
  connecting: 'BRIDGING',
  active: 'CONNECTED',
  error: 'LINK FAILURE'
}[connectionStatus.value]))

// --- Logic ---

/**
 * Handles the high-level bridge toggle.
 * Passes the current tunnelUrl value to the bridge logic.
 */
async function handleToggleConnection(urlValue) {
  if (isConnected.value) return disconnect()
  if (!urlValue || !urlValue.trim()) return

  connectionStatus.value = 'connecting'
  try {
    // 1. Establish the Rust/Electron Bridge
    await window.electron.clientConnect(urlValue)
    connectionStatus.value = 'active'

    // 2. Immediately scan for nodes once bridged
    await refreshRemoteServers()
  } catch (err) {
    console.error("Bridge failed:", err)
    connectionStatus.value = 'error'
  }
}

/**
 * Scans the remote endpoint for active instances.
 */
async function refreshRemoteServers() {
  if (!tunnelUrl.value || isScanning.value) return
  isScanning.value = true

  try {
    const response = await fetch(`${tunnelUrl.value}/api/v1/instances`)
    if (!response.ok) throw new Error('Network response was not ok')

    const data = await response.json()
    remoteInstances.value = Array.isArray(data) ? data : []
  } catch (err) {
    console.error("Discovery failed:", err)
  } finally {
    isScanning.value = false
  }
}

async function disconnect() {
  try {
    await window.electron.killContainers()
  } finally {
    connectionStatus.value = 'idle'
    remoteInstances.value = []
    // Optional: clear URL on disconnect
    // tunnelUrl.value = ''
  }
}

onUnmounted(() => { if (isConnected.value) disconnect() })
</script>

<template>
  <main class="beacon-shell">
    <section class="viewport">
      <header class="view-header">
        <div class="title-stack">
          <p class="breadcrumb">Project Beacon / <span class="highlight">SVG-North Cluster</span></p>
          <h1 class="view-title">Tunnel Gateway</h1>
        </div>
        <div class="header-actions">
          <button v-if="isConnected" class="btn-ghost" @click="refreshRemoteServers">
            <span :class="{ 'spin': isScanning }" class="refresh-icon">↻</span>
            {{ isScanning ? 'Scanning...' : 'Rescan Host' }}
          </button>
        </div>
      </header>

      <div :class="['central-card', { 'card-active': isConnected }]">
        <div class="card-top">
          <div :class="['status-pill', connectionStatus]">
            <span class="status-dot"></span> {{ statusLabel }}
          </div>
          <div class="card-meta">
            <span class="meta-label">GATEWAY ID:</span>
            <span class="meta-value">{{ isConnected ? 'BEACON-B42' : 'OFFLINE' }}</span>
          </div>
        </div>

        <div class="bridge-control">
          <label class="input-label">Remote Edge Endpoint</label>
          <div class="input-group">
            <input
              v-model="tunnelUrl"
              placeholder="https://tunnel-address.trycloudflare.com"
              class="industrial-input"
              :disabled="isConnected || connectionStatus === 'connecting'"
              @keyup.enter="handleToggleConnection(tunnelUrl)"
            />
            <button
              :class="['btn-action', isConnected ? 'btn-danger' : 'btn-primary']"
              @click="handleToggleConnection(tunnelUrl)"
              :disabled="connectionStatus === 'connecting' || (!tunnelUrl && !isConnected)"
            >
              <template v-if="connectionStatus === 'connecting'">
                <span class="spin">⟳</span>
              </template>
              <template v-else>
                {{ isConnected ? 'TERMINATE' : 'ESTABLISH' }}
              </template>
            </button>
          </div>
        </div>
      </div>

      <div v-if="remoteInstances.length > 0" class="node-grid">
        <article v-for="node in remoteInstances" :key="node.id" class="node-card">
          <div class="node-header">
            <span class="node-badge">REMOTE NODE</span>
            <span class="node-ver">v{{ node.version || '1.0' }}</span>
          </div>
          <h3 class="node-name">{{ node.name }}</h3>
          <div class="node-stats">
            <code>PORT: {{ node.port }}</code>
          </div>
          <div class="node-footer">
            <button class="btn-tactile">SYNC</button>
            <button class="btn-tactile btn-secondary">METADATA</button>
          </div>
        </article>
      </div>

      <div v-else class="placeholder-card">
        <div class="placeholder-inner">
          <span class="placeholder-icon">✦</span>
          <h3>{{ isConnected ? 'No Nodes Found' : 'Waiting for Bridge' }}</h3>
          <p>{{ isConnected ? 'The host is connected but no active instances were detected.' : 'Establish a link to synchronize remote instances.' }}</p>
        </div>
      </div>
    </section>
  </main>
</template>

<style scoped>
/* Industrial Dark Palette Style Ref: Beacon Hub */
.beacon-shell {
  display: flex;
  justify-content: center;
  min-height: 100vh;
  background: #141518;
  color: #ffffff;
  font-family: 'Inter', system-ui, sans-serif;
  padding: 40px 20px;
}

.viewport {
  width: 100%;
  max-width: 1000px;
  background: #1e2024;
  border: 1px solid #2d3036;
  border-radius: 32px;
  padding: 48px;
  box-shadow: 0 40px 100px rgba(0,0,0,0.5);
}

.view-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 40px;
}

.view-title {
  font-size: 2.4rem;
  font-weight: 800;
  letter-spacing: -1px;
  margin: 0;
  color: #f0f0f0;
}

.breadcrumb {
  color: #6a6f78;
  font-weight: 600;
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 8px;
}

.highlight { color: #007aff; }

.central-card {
  background: #26292f;
  border: 1px solid #343840;
  border-radius: 24px;
  padding: 32px;
  margin-bottom: 40px;
  transition: all 0.4s cubic-bezier(0.165, 0.84, 0.44, 1);
}

.card-active {
  border-color: #007aff;
  background: #2a2e36;
  box-shadow: 0 0 40px rgba(0, 122, 255, 0.1);
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.status-pill {
  background: #000;
  color: #fff;
  padding: 6px 14px;
  border-radius: 8px;
  font-size: 0.7rem;
  font-weight: 900;
  text-transform: uppercase;
  display: flex;
  align-items: center;
  gap: 8px;
  border: 1px solid #333;
}

.status-dot { width: 8px; height: 8px; background: #444; border-radius: 50%; }

/* Connected Pulse Animation */
.active .status-dot {
  background: #32d74b;
  box-shadow: 0 0 10px #32d74b;
  animation: pulse 2s infinite;
}

.input-label {
  display: block;
  font-size: 0.75rem;
  font-weight: 700;
  color: #8e9297;
  margin-bottom: 12px;
  text-transform: uppercase;
}

.input-group {
  display: flex;
  gap: 12px;
}

.industrial-input {
  flex: 1;
  background: #141518;
  border: 1px solid #3d424a;
  border-radius: 12px;
  padding: 16px 20px;
  color: #fff;
  font-family: 'JetBrains Mono', monospace;
  transition: border 0.2s, box-shadow 0.2s;
}

.industrial-input:focus {
  border-color: #007aff;
  box-shadow: 0 0 0 4px rgba(0, 122, 255, 0.15);
  outline: none;
}

.btn-action {
  padding: 0 32px;
  border-radius: 12px;
  font-weight: 800;
  border: none;
  cursor: pointer;
  transition: transform 0.1s, filter 0.2s;
}

.btn-action:hover:not(:disabled) { filter: brightness(1.1); }
.btn-action:active:not(:disabled) { transform: scale(0.97); }

.btn-primary { background: #007aff; color: #fff; }
.btn-danger { background: #ff3b30; color: #fff; }

.node-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
}

.node-card {
  background: #26292f;
  border: 1px solid #343840;
  border-radius: 20px;
  padding: 24px;
  transition: transform 0.2s;
}

.node-card:hover { transform: translateY(-4px); }

.node-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 16px;
}

.node-badge {
  font-size: 0.65rem;
  font-weight: 900;
  color: #007aff;
  background: rgba(0, 122, 255, 0.1);
  padding: 4px 8px;
  border-radius: 4px;
}

.node-name { margin: 0 0 8px 0; font-size: 1.1rem; font-weight: 700; }
.node-stats { margin-bottom: 20px; opacity: 0.6; font-size: 0.85rem; font-family: 'JetBrains Mono', monospace; }

.btn-tactile {
  background: #eee;
  color: #000;
  border: none;
  padding: 10px 16px;
  border-radius: 8px;
  font-weight: 800;
  font-size: 0.7rem;
  cursor: pointer;
}

.btn-secondary {
  background: transparent;
  color: #fff;
  border: 1px solid #444;
  margin-left: 8px;
}

.placeholder-card {
  border: 2px dashed #343840;
  border-radius: 24px;
  padding: 60px 20px;
  text-align: center;
  color: #6a6f78;
}

.placeholder-icon { font-size: 2rem; display: block; margin-bottom: 16px; }

@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.4; }
  100% { opacity: 1; }
}

.spin { animation: rotation 1s infinite linear; display: inline-block; }
@keyframes rotation { from { transform: rotate(0deg); } to { transform: rotate(359deg); } }

.btn-ghost {
  background: #26292f;
  border: 1px solid #343840;
  color: #8e9297;
  padding: 10px 20px;
  border-radius: 12px;
  cursor: pointer;
  font-weight: 700;
  font-size: 0.85rem;
}
</style>