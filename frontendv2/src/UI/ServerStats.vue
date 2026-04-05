<template>
  <div class="stats-container glass">
    <header class="stats-header">
      <button class="back-btn" @click="$emit('back')">← Back to Cluster</button>
      <div class="title">
        <h2>{{ server.name }}</h2>
        <span class="status-indicator online"></span>
      </div>
    </header>

    <div class="stats-grid">
      <div class="stat-card console">
        <label>Live Logs</label>
        <div class="log-window">
          <p v-for="(log, i) in logs" :key="i"><code>{{ log }}</code></p>
        </div>
      </div>

      <div class="stat-card players">
        <label>Online Players ({{ players.length }})</label>
        <div class="player-list">
          <div v-for="player in players"
               :key="player.uuid"
               class="player-row"
               @click="$emit('inspect-player', player)">
            <img :src="`https://mc-heads.net/avatar/${player.name}/32`" alt="head" />
            <div class="player-meta">
              <span class="p-name">{{ player.name }}</span>
              <span class="p-ping">{{ player.ping }}ms</span>
            </div>
            <span class="chevron">›</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.stats-container { padding: 30px; border-radius: 30px; color: #1d1d1f; }
.stats-header { display: flex; align-items: center; gap: 20px; margin-bottom: 30px; }
.back-btn { background: none; border: none; font-weight: 700; color: #0071e3; cursor: pointer; }

.stats-grid { display: grid; grid-template-columns: 2fr 1fr; gap: 24px; }
.stat-card { background: #f5f5f7; border-radius: 20px; padding: 20px; }

.log-window {
  height: 300px; background: #1d1d1f; border-radius: 12px; padding: 15px;
  overflow-y: auto; color: #32d74b; font-family: 'JetBrains Mono', monospace; font-size: 0.8rem;
}

.player-list { display: flex; flex-direction: column; gap: 10px; margin-top: 15px; }
.player-row {
  display: flex; align-items: center; gap: 12px; background: #fff; padding: 12px;
  border-radius: 14px; cursor: pointer; transition: 0.2s;
}
.player-row:hover { transform: translateX(5px); box-shadow: 0 5px 15px rgba(0,0,0,0.05); }
.p-name { font-weight: 700; display: block; }
.p-ping { font-size: 0.7rem; color: #32d74b; font-weight: 800; }
</style>