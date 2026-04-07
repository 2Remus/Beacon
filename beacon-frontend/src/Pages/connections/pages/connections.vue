<script setup>
import { ref, onMounted, onUnmounted } from 'vue';

const tunnelStatus = ref('disconnected'); // 'connected', 'disconnected', 'error'
const connections = ref([]);
const isUpdating = ref(false);
const errorMsg = ref('');

// Fetch current tunnel state and active users
const fetchTunnelData = async () => {
  isUpdating.value = true;
  try {
    // Calling relative path to bypass CORS via Nginx proxy
    const response = await fetch('/api/v1/tunnel/status');
    if (!response.ok) throw new Error('Failed to fetch tunnel status');

    const data = await response.json();
    tunnelStatus.value = data.status; // e.g., "active"
    connections.value = data.active_connections; // Array of user objects
    errorMsg.value = '';
  } catch (err) {
    errorMsg.value = "Could not reach Project Beacon API.";
    tunnelStatus.value = 'error';
  } finally {
    isUpdating.value = false;
  }
};

const toggleTunnel = async () => {
  const action = tunnelStatus.value === 'active' ? 'stop' : 'start';
  try {
    let res = window.electron.startCloudflare();
    console.log(res);
  } catch (err) {
    errorMsg.value = `Failed to ${action} tunnel.`;
  }
};

let interval;
onMounted(() => {
  fetchTunnelData();
  interval = setInterval(fetchTunnelData, 5000); // Auto-refresh every 5s
});

onUnmounted(() => clearInterval(interval));
</script>

<template>
  <div class="connections-container">
    <header class="status-card" :class="tunnelStatus">
      <div class="info">
        <h1>Cloudflare Tunnel</h1>
        <p v-if="tunnelStatus === 'active'" class="badge">● Online</p>
        <p v-else class="badge">○ Offline</p>
      </div>
      <button @click="toggleTunnel" :disabled="isUpdating">
        {{ tunnelStatus === 'active' ? 'Disconnect' : 'Connect' }}
      </button>
    </header>

    <div v-if="errorMsg" class="error-banner">{{ errorMsg }}</div>

    <section class="connections-list">
      <h2>Active Connections ({{ connections.length }})</h2>
      <table>
        <thead>
        <tr>
          <th>User</th>
          <th>IP Address</th>
          <th>Latency</th>
          <th>Uptime</th>
        </tr>
        </thead>
        <tbody>
        <tr v-for="user in connections" :key="user.id">
          <td>
            <div class="user-cell">
              <img :src="user.avatar || 'https://via.placeholder.com/32'" alt="">
              {{ user.name }}
            </div>
          </td>
          <td><code>{{ user.ip }}</code></td>
          <td>{{ user.ping }}ms</td>
          <td>{{ user.duration }}</td>
        </tr>
        <tr v-if="connections.length === 0">
          <td colspan="4" class="empty">No active users via Tunnel.</td>
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

.empty { text-align: center; padding: 3rem; color: #64748b; }
</style>