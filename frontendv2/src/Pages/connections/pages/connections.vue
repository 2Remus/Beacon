<script setup>
import { ref, onMounted, onUnmounted } from 'vue';

const tunnelStatus = ref('offline'); // 'RUNNING', 'offline', 'error'
const connections = ref([]);
const isUpdating = ref(false);
const errorMsg = ref('');
const publicUrl = ref(''); // To store the cleaned .trycloudflare.com link

// Helper to extract the URL from the messy Cloudflare log string
const cleanUrl = (rawUrl) => {
  if (!rawUrl) return '';
  const match = rawUrl.match(/https:\/\/[a-z0-9-]+\.trycloudflare\.com/);
  return match ? match[0] : rawUrl;
};

const toggleTunnel = async () => {
  isUpdating.value = true;
  if (tunnelStatus.value === 'offline') {
    try {
      // Note: ensure the method name matches your preload.js (startCloudflare vs startCloudflared)
      const res = await window.electron.startCloudflare(25565);

      console.log('Tunnel Response:', res);

      // Update refs with data from the Proxy object
      tunnelStatus.value = res.status; // "RUNNING"
      publicUrl.value = cleanUrl(res.url);
      connections.value = res.connections || [];
      errorMsg.value = '';
    } catch (err) {
      console.error('Tunnel Error:', err);
      errorMsg.value = "Failed to launch sidecar binary.";
      tunnelStatus.value = 'error';
    } finally {
      isUpdating.value = false;
    }
  }
  else{
    try{
      window.electron.stopCloudflare();
      tunnelStatus.value = "offline"; // "RUNNING"
      publicUrl.value = "";
      connections.value =  [];
    }catch(err){
      console.error('Tunnel Error:', err);
    }finally {
      isUpdating.value = false;
    }
  }

};

// If you have a separate API for stats, keep this, otherwise we rely on the Electron toggle
const fetchTunnelData = async () => {
  // If the tunnel isn't running, don't bother polling the local API
  if (tunnelStatus.value !== 'RUNNING') return;

  try {
    const response = await fetch('/api/v1/tunnel/status');
    if (response.ok) {
      const data = await response.json();
      connections.value = data.active_connections;
    }
  } catch (err) {
    // Silently fail polling to avoid UI flicker
  }
};

let interval;
onMounted(() => {
  interval = setInterval(fetchTunnelData, 5000);
});

onUnmounted(() => clearInterval(interval));
</script>

<template>
  <div class="connections-container">
    <header class="status-card" :class="{ 'active': tunnelStatus === 'RUNNING', 'error': tunnelStatus === 'error' }">
      <div class="info">
        <h1>Cloudflare Tunnel</h1>
        <div v-if="tunnelStatus === 'RUNNING'">
          <p class="badge">● Online</p>
          <code class="url-display">{{ publicUrl }}</code>
        </div>
        <p v-else class="badge offline">○ Offline</p>
      </div>
      <button @click="toggleTunnel" :disabled="isUpdating" :class="{ 'btn-stop': tunnelStatus === 'RUNNING' }">
        {{ isUpdating ? 'Processing...' : (tunnelStatus === 'RUNNING' ? 'Disconnect' : 'Connect') }}
      </button>
    </header>

    <div v-if="errorMsg" class="error-banner">{{ errorMsg }}</div>

    <section class="connections-list">
      <h2>Active Edges ({{ connections.length }})</h2>
      <table>
        <thead>
        <tr>
          <th>Node</th>
          <th>Type</th>
          <th>Status</th>
        </tr>
        </thead>
        <tbody>
        <tr v-for="(conn, index) in connections" :key="index">
          <td>
            <div class="user-cell">
              <span class="icon">🌐</span>
              {{ conn }}
            </div>
          </td>
          <td><code>Anycast</code></td>
          <td><span class="status-text">Connected</span></td>
        </tr>
        <tr v-if="connections.length === 0">
          <td colspan="3" class="empty">No active connections. Click connect to initialize.</td>
        </tr>
        </tbody>
      </table>
    </section>
  </div>
</template>

<style scoped>
.connections-container {
  padding: 2rem;
  color: #e2e8f0;
}

.status-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem;
  border-radius: 12px;
  background: #1e293b;
  border-left: 6px solid #64748b;
  margin-bottom: 2rem;
}

.status-card.active { border-left-color: #10b981; }
.status-card.error { border-left-color: #ef4444; }

.badge {
  font-weight: bold;
  color: #10b981;
  margin-top: 0.5rem;
}

button {
  background: #3b82f6;
  color: white;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 600;
  transition: opacity 0.2s;
}

button:disabled { opacity: 0.5; cursor: not-allowed; }

table {
  width: 100%;
  border-collapse: collapse;
  background: #0f172a;
  border-radius: 8px;
  overflow: hidden;
}

th { text-align: left; padding: 1rem; background: #1e293b; color: #94a3b8; }
td { padding: 1rem; border-top: 1px solid #1e293b; }

.user-cell { display: flex; align-items: center; gap: 0.75rem; }
.user-cell img { border-radius: 50%; width: 32px; height: 32px; }

.error-banner {
  background: #7f1d1d;
  padding: 1rem;
  border-radius: 8px;
  margin-bottom: 1rem;
}


/* Add these to your existing styles */

.url-display {
  display: block;
  margin-top: 10px;
  background: rgba(0, 0, 0, 0.3);
  padding: 8px 12px;
  border-radius: 6px;
  color: #3b82f6; /* Beacon Blue */
  font-family: 'Fira Code', monospace;
  font-size: 0.85rem;
  border: 1px solid rgba(59, 130, 246, 0.2);
}

.badge.offline {
  color: #64748b;
}

.btn-stop {
  background: #ef4444 !important;
}

.status-text {
  color: #10b981;
  font-size: 0.8rem;
  text-transform: uppercase;
  font-weight: bold;
}

.icon {
  font-size: 1.2rem;
}

.empty { text-align: center; padding: 3rem; color: #64748b; }
</style>