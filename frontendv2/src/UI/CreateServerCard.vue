<template>
  <div class="modal-body">
    <div class="form-section">
      <label class="section-label">Server Identity</label>
      <div class="input-stack glass">
        <input v-model="config.name" type="text" placeholder="Server Name (e.g., Survival Hub)" class="fancy-input main" />
        <div class="divider"></div>
        <input v-model="config.motd" type="text" placeholder="Message of the Day (MOTD)" class="fancy-input sub" />
      </div>
    </div>

    <div class="form-section">
      <label class="section-label">Platform & Version</label>
      <div class="platform-grid">
        <div v-for="type in platforms"
             :key="type.id"
             class="platform-card"
             :class="{ active: config.type === type.name }"
             @click="config.type = type.name">
          <img :src="type.icon" :alt="type.name" class="platform-logo" />
          <span class="platform-name">{{ type.name }}</span>
        </div>
      </div>

      <div class="custom-select-wrapper">
        <select v-model="config.version" class="fancy-select">
          <optgroup label="Latest Stability">
            <option>1.21.1</option>
            <option>1.20.1</option>
          </optgroup>
          <optgroup label="Legacy / Modding">
            <option>1.19.4</option>
            <option>1.12.2</option>
            <option>1.8.9</option>
          </optgroup>
        </select>
      </div>
    </div>

    <div class="form-section">
      <label class="section-label">Gameplay Difficulty</label>
      <div class="segmented-control glass">
        <button v-for="level in ['Peaceful', 'Easy', 'Normal', 'Hard']"
                :key="level"
                :class="{ active: config.difficulty === level }"
                @click="config.difficulty = level">
          {{ level }}
        </button>
      </div>
    </div>

    <footer class="modal-footer">
      <button class="btn-cancel" @click="$emit('close')">Cancel</button>
      <button class="btn-deploy" @click="handleDeploy">
        <span class="sparkle">✦</span> Initialize Deployment
      </button>
    </footer>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const platforms = [
  { id: 1, name: 'Vanilla', icon: 'https://p7.hiclipart.com/preview/835/10/446/minecraft-grass-block-video-game-pixel-art-grass.jpg' },
  { id: 2, name: 'Spigot', icon: 'https://static.spigotmc.org/styles/spigot/favicon.ico' },
  { id: 3, name: 'Forge', icon: 'https://files.minecraftforge.net/static/logo.png' },
  { id: 4, name: 'Fabric', icon: 'https://fabricmc.net/assets/logo.png' }
]

const config = ref({
  name: '',
  motd: 'A Project Beacon Server',
  version: '1.21.1',
  type: 'Vanilla',
  difficulty: 'Normal'
})

const handleDeploy = () => {
  console.log("Pushing to Cluster:", config.value)
}
</script>

<style scoped>
.modal-body { display: flex; flex-direction: column; gap: 32px; padding: 10px; }
.section-label { display: block; font-size: 0.7rem; font-weight: 900; color: #86868b; text-transform: uppercase; margin-bottom: 12px; letter-spacing: 1px; }

/* Fancy Input Stack */
.input-stack.glass { background: #f5f5f7; border-radius: 20px; overflow: hidden; border: 1px solid rgba(0,0,0,0.03); }
.fancy-input { width: 100%; border: none; background: transparent; padding: 18px 24px; outline: none; transition: 0.2s; }
.fancy-input.main { font-size: 1.1rem; font-weight: 700; color: #1d1d1f; }
.fancy-input.sub { font-size: 0.9rem; color: #86868b; }
.divider { height: 1px; background: rgba(0,0,0,0.05); margin: 0 20px; }

/* Platform Cards */
.platform-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 16px; }
.platform-card {
  background: #f5f5f7;
  padding: 16px;
  border-radius: 18px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  border: 2px solid transparent;
}
.platform-card:hover { transform: translateY(-3px); background: #fff; box-shadow: 0 10px 20px rgba(0,0,0,0.05); }
.platform-card.active { border-color: #0071e3; background: #fff; box-shadow: 0 10px 25px rgba(0,113,227,0.1); }
.platform-logo { width: 32px; height: 32px; object-fit: contain; filter: grayscale(100%); transition: 0.3s; }
.platform-card.active .platform-logo { filter: grayscale(0%); }
.platform-name { font-size: 0.75rem; font-weight: 800; color: #86868b; }
.platform-card.active .platform-name { color: #0071e3; }

/* Fancy Select */
.fancy-select {
  width: 100%;
  padding: 16px 20px;
  background: #f5f5f7;
  border-radius: 16px;
  border: none;
  font-weight: 700;
  color: #1d1d1f;
  appearance: none;
  background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='black' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e");
  background-repeat: no-repeat;
  background-position: right 20px center;
  background-size: 16px;
}

/* Segmented Control */
.segmented-control.glass { background: #f5f5f7; padding: 6px; border-radius: 16px; display: flex; gap: 4px; }
.segmented-control button {
  flex: 1; padding: 12px; border-radius: 12px; border: none; background: transparent;
  font-size: 0.8rem; font-weight: 700; color: #86868b; cursor: pointer; transition: 0.2s;
}
.segmented-control button.active { background: #fff; color: #0071e3; box-shadow: 0 4px 12px rgba(0,0,0,0.08); }

/* Footer */
.modal-footer { display: flex; justify-content: flex-end; align-items: center; gap: 20px; margin-top: 20px; }
.btn-cancel { background: none; border: none; font-weight: 700; color: #86868b; cursor: pointer; }
.btn-deploy {
  background: #0071e3; color: white; border: none; padding: 16px 32px; border-radius: 18px;
  font-weight: 800; cursor: pointer; transition: transform 0.2s; display: flex; align-items: center; gap: 10px;
}
.btn-deploy:hover { transform: scale(1.02); background: #0077ed; }
.sparkle { color: #52e5ff; }
</style>