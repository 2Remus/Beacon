<template>
  <div class="player-overlay glass" @click.self="$emit('close')">
    <div class="player-modal">
      <div class="player-profile">
        <img :src="`https://mc-heads.net/body/${player.name}/120`" class="player-body" />
        <div class="profile-text">
          <h2>{{ player.name }}</h2>
          <p>Level {{ player.level }} Explorer</p>
        </div>
      </div>

      <div class="vitals">
        <div class="vital-row">
          <label>Health</label>
          <div class="heart-container">
            <span v-for="i in 10" :key="i" class="heart" :class="{ empty: i > player.health / 2 }">❤️</span>
          </div>
        </div>

        <div class="vital-row">
          <label>Experience ({{ player.xp }}%)</label>
          <div class="xp-bar"><div class="xp-fill" :style="{ width: player.xp + '%' }"></div></div>
        </div>
      </div>

      <div class="inventory-grid">
        <label>Hotbar Snapshot</label>
        <div class="slots">
          <div v-for="i in 9" :key="i" class="slot">
            <img v-if="player.inventory[i-1]" :src="player.inventory[i-1].icon" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.player-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.2); display: flex; align-items: center; justify-content: center; z-index: 2000; }
.player-modal { background: #fff; width: 450px; padding: 40px; border-radius: 35px; box-shadow: 0 30px 60px rgba(0,0,0,0.15); }

.player-profile { display: flex; align-items: center; gap: 20px; margin-bottom: 30px; }
.player-body { height: 120px; filter: drop-shadow(0 10px 10px rgba(0,0,0,0.1)); }

.heart-container { display: flex; gap: 2px; font-size: 1.2rem; }
.heart.empty { filter: grayscale(1) opacity(0.3); }

.xp-bar { height: 8px; background: #eee; border-radius: 10px; overflow: hidden; margin-top: 5px; }
.xp-fill { height: 100%; background: #32d74b; box-shadow: 0 0 10px #32d74b; }

.slots { display: grid; grid-template-columns: repeat(9, 1fr); gap: 5px; margin-top: 10px; }
.slot { width: 40px; height: 40px; background: #f5f5f7; border-radius: 8px; display: flex; align-items: center; justify-content: center; border: 1px solid #eee; }
.slot img { width: 24px; }
</style>