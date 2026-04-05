<script setup>
import { ref, onMounted, onUnmounted } from 'vue';

const containers = ref([]);
const loading = ref(true);
const error = ref(null);

// Fetch containers from your Rust backend
const fetchContainers = async () => {
  try {
    // Relative path works because Nginx proxies /api/v1 to Rust
    const response = await fetch('/api/v1/docker/containers');
    if (!response.ok) throw new Error('Failed to fetch Docker stats');

    const data = await response.json();
    containers.value = data;
    error.value = null;
  } catch (err) {
    error.value = "Unable to connect to Docker Engine via Beacon API.";
    console.error(err);
  } finally {
    loading.value = false;
  }
};

const getStatusClass = (status) => {
  if (status.toLowerCase().includes('up')) return 'status-up';
  if (status.toLowerCase().includes('exited')) return 'status-down';
  return 'status-paused';
};

let poll;
onMounted(() => {
  fetchContainers();
  poll = setInterval(fetchContainers, 3000); // Live updates every 3s
});

onUnmounted(() => clearInterval(poll));

const toggleContainer = async (id, currentStatus) => {
  const action = currentStatus.includes('Up') ? 'stop' : 'start';
  try {
    await fetch(`/api/v1/docker/containers/${id}/${action}`, { method: 'POST' });
    fetchContainers(); // Refresh immediately
  } catch (err) {
    alert(`Failed to ${action} container`);
  }
};
</script>

<template>
  <div class="docker-dashboard">
    <header class="header">
      <h2>System Infrastructure</h2>
      <div v-if="loading" class="loader">Scanning Stack...</div>
    </header>

    <div v-if="error" class="error-msg">{{ error }}</div>

    <div class="container-grid">
      <div v-for="container in containers" :key="container.id" class="card">
        <div class="card-header">
          <span class="indicator" :class="getStatusClass(container.status)"></span>
          <h3>{{ container.name }}</h3>
        </div>

        <div class="card-body">
          <p class="image-name">{{ container.image }}</p>
          <div class="stats">
            <div class="stat">
              <label>Status</label>
              <span>{{ container.status }}</span>
            </div>
            <div class="stat">
              <label>Ports</label>
              <span>{{ container.ports || 'None' }}</span>
            </div>
          </div>
        </div>

        <div class="card-actions">
          <button @click="toggleContainer(container.id, container.status)" :class="container.status.includes('Up') ? 'btn-stop' : 'btn-start'">
            {{ container.status.includes('Up') ? 'Stop' : 'Start' }}
          </button>
          <button class="btn-logs">Logs</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.docker-dashboard {
  padding: 20px;
  color: #f8fafc;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
}

.container-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1.5rem;
}

.card {
  background: #1e293b;
  border: 1px solid #334155;
  border-radius: 12px;
  padding: 1.5rem;
  transition: transform 0.2s, border-color 0.2s;
}

.card:hover {
  border-color: #3b82f6;
  transform: translateY(-2px);
}

.card-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 1rem;
}

.indicator {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.status-up { background: #10b981; box-shadow: 0 0 8px #10b981; }
.status-down { background: #ef4444; }
.status-paused { background: #f59e0b; }

.image-name {
  font-size: 0.8rem;
  color: #94a3b8;
  margin-bottom: 1rem;
  font-family: monospace;
}

.stats {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  font-size: 0.85rem;
}

.stat label {
  display: block;
  color: #64748b;
  font-size: 0.7rem;
  text-transform: uppercase;
}

.card-actions {
  margin-top: 1.5rem;
  display: flex;
  gap: 10px;
}

button {
  flex: 1;
  padding: 8px;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  font-weight: 600;
  font-size: 0.8rem;
}

.btn-start { background: #059669; color: white; }
.btn-stop { background: #dc2626; color: white; }
.btn-logs { background: #475569; color: white; }

.error-msg {
  background: #450a0a;
  color: #fca5a5;
  padding: 1rem;
  border-radius: 8px;
  margin-bottom: 2rem;
}
</style>