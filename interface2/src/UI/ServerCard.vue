<template>
  <div class="server-card">
    <div class="card-glow" :class="{ 'glow-active': server.status === 'online' }"></div>

    <div class="card-content">
      <div class="card-header">
        <div class="status-indicator">
          <span class="pulse-dot" :class="server.status"></span>
          <span class="status-text">{{ server.status }}</span>
        </div>
        <button class="settings-gear">⚙</button>
      </div>

      <h3 class="server-name">{{ server.name }}</h3>
      <p class="server-addr">{{ server.address }}</p>

      <div class="usage-stats">
        <div class="stat-row">
          <span>CPU</span>
          <span>{{ server.cpu }}%</span>
        </div>
        <div class="progress-track">
          <div class="progress-fill" :style="{ width: server.cpu + '%' }"></div>
        </div>
      </div>

      <div class="card-actions">
        <button v-if="server.status !== 'online'" class="btn btn-start">START</button>
        <button v-else class="btn btn-stop">STOP</button>
        <button class="btn btn-console">CONSOLE</button>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  server: {
    type: Object,
    required: true
  }
})
</script>

<style scoped>
.server-card {
  position: relative;
  background: rgba(255, 255, 255, 0.7);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.5);
  border-radius: 20px;
  padding: 24px;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.server-card:hover {
  transform: translateY(-8px);
  background: rgba(255, 255, 255, 0.9);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.05);
}

.card-glow {
  position: absolute;
  top: -50%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: radial-gradient(circle, rgba(0, 171, 240, 0.1) 0%, transparent 70%);
  opacity: 0;
  transition: opacity 0.5s;
}

.glow-active { opacity: 1; }

.card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px; }

.pulse-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  display: inline-block;
  margin-right: 8px;
}

.pulse-dot.online { background: #4caf50; box-shadow: 0 0 10px #4caf50; animation: pulse 2s infinite; }
.pulse-dot.offline { background: #e91e63; }

@keyframes pulse {
  0% { transform: scale(0.95); opacity: 0.9; }
  70% { transform: scale(1.1); opacity: 0.4; }
  100% { transform: scale(0.95); opacity: 0.9; }
}

.server-name { font-size: 1.2rem; font-weight: 800; color: #1e2124; margin: 0; }
.server-addr { font-size: 0.85rem; color: #00abf0; font-family: 'JetBrains Mono', monospace; margin-top: 4px; }

.usage-stats { margin: 20px 0; }
.stat-row { display: flex; justify-content: space-between; font-size: 0.75rem; font-weight: 700; color: #666; margin-bottom: 6px; }

.progress-track { height: 6px; background: #eee; border-radius: 10px; overflow: hidden; }
.progress-fill { height: 100%; background: #00abf0; transition: width 1s ease; }

.card-actions { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-top: 20px; }
.btn { border: none; padding: 10px; border-radius: 8px; font-weight: 800; cursor: pointer; font-size: 0.75rem; }
.btn-start { background: #00abf0; color: white; }
.btn-stop { background: #e91e63; color: white; }
.btn-console { background: #f0f2f5; color: #666; }
</style>