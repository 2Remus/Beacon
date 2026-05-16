<script setup>
import { onMounted, ref, onUnmounted, nextTick, computed, watch, reactive } from 'vue'

/**
 * --- PERSISTENCE & HISTORY STORAGE ---
 * Bridges Vue's reactive state with browser sessionStorage.
 */
const STORAGE_KEY = 'BEACON_SESSION_LOGS';

const saveToHistory = (id, data) => {
  try {
    const history = JSON.parse(sessionStorage.getItem(STORAGE_KEY) || '{}');
    history[id] = data;
    sessionStorage.setItem(STORAGE_KEY, JSON.stringify(history));
  } catch (e) { console.error("History Save Error:", e); }
};

const loadFromHistory = (id) => {
  try {
    const history = JSON.parse(sessionStorage.getItem(STORAGE_KEY) || '{}');
    return history[id] || [];
  } catch (e) { return []; }
};

// Initialize the global store window object if it doesn't exist
if (!window.__BEACON_STORE__) {
  window.__BEACON_STORE__ = reactive({
    logs: {},
    activeListeners: new Set()
  });
}
const store = window.__BEACON_STORE__;

const isLoading = ref(true)
const error = ref(null)
const instances = ref([])
const currentView = ref("import")
const activeInstance = ref(null)
const activeMenuId = ref(null)
const isDeploying = ref(false)
const insDragging = ref(false)

// UI Deployment States
const isCreating = ref(false)
const newInstanceName = ref('')
const newInstanceVersion = ref('1.20.1')
const selectedType = ref('Vanilla')
const memoryAlloc = ref('3G')
const isOnlineMode = ref(false)
const selectedFile = ref(null);
const selectedPath = ref("");

const serverTypes = [
  { id: 'Vanilla', name: 'Vanilla', logo: '🍦', desc: 'Official Mojang Engine' },
  { id: 'Fabric', name: 'Fabric', logo: '🧶', desc: 'Lightweight Modding' },
  { id: 'Paper', name: 'Paper', logo: '📜', desc: 'High-Performance Spigot' },
  { id: 'Forge', name: 'Forge', logo: '⚒️', desc: 'Heavy Modding Support' }
]

// Version Utilities
const generateVersions = () => {
  const versions = [];
  for (let i = 1; i >= 0; i--) versions.push(`26.${i}`);
  for (let i = 21; i >= 8; i--) {
    if (i === 8) versions.push("1.8.9");
    else versions.push(`1.${i}.1`);
  }
  return [...new Set(versions)];
};

const allVersions = generateVersions();
const compatibilityMap = {
  Vanilla: allVersions,
  Paper: allVersions.filter(v => !v.startsWith('26')),
  Fabric: allVersions.filter(v => parseFloat(v.split('.')[1]) >= 14 || v.startsWith('26')),
  Forge: allVersions.filter(v => parseFloat(v.split('.')[1]) >= 12 && !v.startsWith('26'))
};

const availableVersions = computed(() => compatibilityMap[selectedType.value] || []);
const isFormValid = computed(() => newInstanceName.value.trim().length > 0 && !isDeploying.value);

watch(selectedType, (newType) => {
  if (!compatibilityMap[newType].includes(newInstanceVersion.value)) {
    newInstanceVersion.value = compatibilityMap[newType][0];
  }
});

const logs = computed(() => {
  if (!activeInstance.value) return [];
  return store.logs[activeInstance.value.id] || [];
});

const logContainer = ref(null)
const selectedPlayer = ref(null)

/**
 * 1. Persistent Log Stream Connection with History Sync
 */
const attachLogListener = async (id) => {
  // Hydrate from sessionStorage if the memory store is empty
  if (!store.logs[id] || store.logs[id].length === 0) {
    store.logs[id] = loadFromHistory(id);
  }

  if (!store.activeListeners.has(id)) {
    window.electron.onLogUpdate(id, (line) => {
      if (!store.logs[id]) store.logs[id] = [];
      store.logs[id].push(line);

      if (store.logs[id].length > 1000) store.logs[id].shift();

      // Throttled persistence to session history
      // We use a small timeout to avoid hammering storage on every log line
      clearTimeout(window[`_sync_${id}`]);
      window[`_sync_${id}`] = setTimeout(() => {
        saveToHistory(id, store.logs[id]);
      }, 500);

      if (currentView.value === 'stats' && activeInstance.value?.id === id) {
        scrollToBottom();
      }
    });
    store.activeListeners.add(id);
  }

  try {
    await window.electron.getLogs(id);
  } catch (err) {
    console.error("Backend log pipe error:", err);
  }
};

/**
 * 2. Cluster Sync
 */
/**
 * 2. Cluster Sync with State Preservation
 */
const fetchServers = async () => {
  isLoading.value = true;
  try {
    const freshData = await window.electron.getServers();

    // Instead of instances.value = freshData, we map and preserve
    instances.value = freshData.map(newServer => {
      // Look for the existing version of this server in our current state
      const existing = instances.value.find(s => s.id === newServer.id);

      if (existing) {
        // Update the existing object properties but keep the reference
        // This prevents Vue from "flickering" or losing local UI states
        return { ...existing, ...newServer };
      }
      return newServer;
    });

    // Re-attach listeners only for servers that aren't already being tracked
    instances.value.forEach(server => {
      if (server.status?.toUpperCase() === 'RUNNING') {
        attachLogListener(server.id);
      }
    });

    error.value = null;
  } catch (err) {
    console.error("Fetch Error:", err);
    error.value = "Failed to synchronize cluster state.";
  } finally {
    isLoading.value = false;
  }
};

const scrollToBottom = async () => {
  await nextTick();
  if (logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight;
  }
};

/**
 * 3. UI Actions
 */

const handleDrop = async(e) => {
  this.insDragging = false
  const files = e.dataTransfer.files;
    if (files.length > 0) {
      this.processFile(files[0]);
      console.logs(files)
    }
}



const triggerImport = async() => {
  console.log(newInstanceName.value,newInstanceVersion.value, selectedType.value, isOnlineMode.value, selectedPath.value)
  try{
     const result =  await window.electron.serverImport(crypto.randomUUID(),newInstanceName.value,newInstanceVersion.value, selectedType.value, isOnlineMode.value, selectedPath.value);
     if (result && !result.error){
       isCreating.value = false;
        newInstanceName.value = '';
        await fetchServers();
     }
  }finally{
    isDeploying.value = false
  }
}

 // const [fileHandle] = await window.showOpenFilePicker({
 //      types: [{
 //        description: 'Server Config (JSON)',
 //        accept: { 'application/zip': ['.zip'] },
 //      }],
 //      multiple: false


const updateSelection = (filePath) => {
  const fileName = filePath.split(/[\\/]/).pop() || "";
  selectedFile.value = {
    name: fileName,
    path: filePath
  };

  selectedPath.value = filePath;

  console.log("Selected File Object:", selectedFile.value);

  // 4. Automatically set the instance name if it's empty
  if (!newInstanceName.value && fileName) {
    newInstanceName.value = fileName.replace(".zip", "");
  }
};


const browseFiles = async () => {
  try {
    // 1. Open the native OS picker via your working Electron bridge
    const filePath = await window.electron.openZipPicker();

    // 2. If the user didn't cancel and a path was returned, pass it to your selection handler
    if (filePath) {
      updateSelection(filePath);
    }
    
  } catch (err) {
    // Catch-all for unexpected IPC or OS errors
    console.error("Failed to select file:", err);
  }
};


const handleFileDrop = (e) => {
  isDragging.value = false;
  const file = e.dataTransfer.files[0];
  
  if (file && file.name.endsWith('.zip')) {
    updateSelection(file);
  }
};



const openServerStats = (server) => {
  activeInstance.value = server;
  currentView.value = "stats";
  attachLogListener(server.id);
  scrollToBottom();
};

const closeMenu = () => {
  currentView.value = "grid";
  activeInstance.value = null;
  selectedPlayer.value = null;
};

const toggleStatus = async (server) => {
  if (!server) return;
  const path = server.instance_path || server.instancePath;

  try {
    if (server.status?.toUpperCase() === 'RUNNING') {
      server.status = 'STOPPING';
      await window.electron.stopServer(server.id.toString());
      server.status = 'STOPPED';
      // Clear history on hard stop if desired
      store.logs[server.id] = [];
      saveToHistory(server.id, []);
    } else {
      server.status = 'STARTING';
      const result = await window.electron.startServer({
        id: server.id.toString(),
        bin_dir: path.toString(),
        ram: parseInt(server.ram) || 3072
      });

      if (result && !result.error) {
        server.status = 'RUNNING';
        attachLogListener(server.id);
      } else {
        server.status = 'ERROR';
      }
    }
  } catch (err) {
    server.status = 'ERROR';
  }
};

const deployInstance = async () => {
  if (!isFormValid.value) return;
  isDeploying.value = true;
  try {
    const result = await window.electron.createServer(
      crypto.randomUUID(), newInstanceName.value, selectedType.value,
      newInstanceVersion.value, memoryAlloc.value, 25565, isOnlineMode.value, isOnlineMode.value
    );
    if (result && !result.error) {
      isCreating.value = false;
      newInstanceName.value = '';
      await fetchServers();
    }
  } finally {
    isDeploying.value = false;
  }
};

const deleteInstance = async (id) => {
  if (!confirm("Permanently delete node?")) return;
  try {
    delete store.logs[id];
    store.activeListeners.delete(id);
    saveToHistory(id, null); // Wipe from storage

    const response = await fetch('http://api.beacon.local/api/v1/servers/delete', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id })
    });
    if (response.ok) {
      activeMenuId.value = null;
      instances.value = instances.value.filter(s => s.id !== id);
    }
  } catch (err) {
    console.error(err);
  }
};

const toggleMenu = (id) => activeMenuId.value = activeMenuId.value === id ? null : id;
const closeMenus = () => activeMenuId.value = null;

onMounted(async () => {
  if (instances.value.length > 0) {
    isLoading.value = false;
  }

  await fetchServers();
  window.addEventListener('click', closeMenus);
});

onUnmounted(() => {
  window.removeEventListener('click', closeMenus);
});
</script>

<template>
  <div class="dashboard-container">

    <transition name="pop">
      <div v-if="isCreating" class="modal-overlay" @click.self="isCreating = false">
        <div class="modal-glass">
          <span class="section-label">Initialize Node</span>
          <h2 class="deploy-instance">Deploy New Instance</h2>

          <div class="input-stack">
            <label>Node Identity</label>
            <input
              v-model="newInstanceName"
              class="fancy-input"
              placeholder="e.g. survival-hub-01"
              autofocus
            />
          </div>

          <div class="input-stack">
            <label>Server Software</label>
            <div class="type-grid">
              <div
                v-for="type in serverTypes"
                :key="type.id"
                class="type-box"
                :class="{ active: selectedType === type.id }"
                @click="selectedType = type.id"
              >
                <div class="type-logo">{{ type.logo }}</div>
                <div class="type-info">
                  <span class="type-name">{{ type.name }}</span>
                  <span class="type-desc">{{ type.desc }}</span>
                </div>
              </div>
            </div>
          </div>

          <div class="form-row">
            <div class="input-stack half">
              <label>Engine Version</label>
              <select v-model="newInstanceVersion" class="fancy-input">
                <option v-for="v in availableVersions" :key="v" :value="v">{{ v }}</option>
              </select>
            </div>
            <div class="input-stack half">
              <label>RAM Allocation</label>
              <select v-model="memoryAlloc" class="fancy-input">
                <option>2G</option>
                <option>3G</option>
                <option>4G</option>
                <option>8G</option>
              </select>
            </div>
          </div>

          <div class="toggle-stack" @click="isOnlineMode = !isOnlineMode">
            <div class="toggle-info">
              <span class="toggle-label">Online Mode</span>
              <span class="toggle-desc">Require official Mojang authentication</span>
            </div>
            <div class="toggle-switch" :class="{ on: isOnlineMode }"></div>
          </div>

          <div class="modal-actions">
            <button class="btn-secondary" @click="isCreating = false">Cancel</button>
            <button
              class="primary-btn sparkle-hover"
              :disabled="!isFormValid"
              @click="deployInstance"
            >
              {{ isDeploying ? 'Provisioning...' : 'Start Deployment' }}
            </button>
          </div>
        </div>
      </div>
    </transition>

    <div v-if="currentView === 'grid'" class="view-layer">
      <header class="dashboard-header">
        <div class="title-section">
          <h1>Cloud Instances</h1>
          <p>Project Beacon / <span>SVG-North Cluster</span></p>
        </div>
        <button class="primary-btn sparkle-hover" @click="isCreating = true">
          <span class="plus">✦</span> New Instance
        </button>
      </header>

      <div v-if="isLoading" class="status-msg">Synchronizing Cluster Data...</div>
      <div v-else-if="error" class="status-msg error">{{ error }}</div>

      <div v-else class="server-grid">
        <div
          v-for="server in instances"
          :key="server.id"
          class="card server-card"
          :class="server.status?.toLowerCase()"
          @click="openServerStats(server)"
        >
          <div class="card-inner">
            <div class="card-head">
              <div class="status-pill">
                <span class="dot"></span> {{ server.status }}
              </div>

              <div class="context-container" @click.stop>
                <button class="btn-dots" @click="toggleMenu(server.id)">•••</button>
                <transition name="pop">
                  <div v-if="activeMenuId === server.id" class="dropdown-menu">
                    <button class="menu-item delete" @click="deleteInstance(server.id)">
                      <span class="icon">🗑️</span> Delete Instance
                    </button>
                  </div>
                </transition>
              </div>

              <div class="badge-group">
                <span class="version-badge">{{ server.version }}</span>
                <span class="type-badge">{{ server.type }}</span>
              </div>
            </div>

            <div class="server-info">
              <h3>{{ server.name }}</h3>
              <code>{{ server.address }}</code>
            </div>

            <div class="metrics">
              <div class="metric-labels"><span>CPU Usage</span><span>{{ server.cpu_usage || 0 }}%</span></div>
              <div class="progress-bg"><div class="progress-fill" :style="{ width: (server.cpu_usage || 0) + '%' }"></div></div>
            </div>

            <div class="card-actions" @click.stop>
              <button
                class="btn-action"
                :class="server.status?.toLowerCase()"
                @click="toggleStatus(server)"
              >
                {{ server.status === 'RUNNING' ? 'STOP' : 'START' }}
              </button>
              <button class="btn-secondary" @click="openServerStats(server)">CONSOLE</button>
            </div>
          </div>
        </div>

        <div class="card create-card" @click="isCreating = true">
          <div class="plus-icon">✦</div>
          <div class="create-text">
            <h3>Deploy Instance</h3>
            <p>Initialize a new node</p>
          </div>
        </div>
      </div>
    </div>

    <div v-if="currentView === 'stats'" class="view-layer stats-view-container">
      <header class="dashboard-header">
        <div class="title-section">
          <h1>Instance Control</h1>
          <p>Project Beacon / <span class="active-node">{{ activeInstance?.name }}</span></p>
        </div>

        <div class="header-actions">
          <button class="btn-secondary" @click="closeMenu">Return to Dashboard</button>
        </div>
      </header>

      <div class="stats-layout">
        <div class="terminal-column">
          <span class="section-label">Live System Output</span>
          <div class="log-terminal" ref="logContainer">
            <div v-for="(line, index) in logs" :key="index" class="log-line">
              <span class="line-number">{{ index + 1 }}</span>
              <span class="content">{{ line }}</span>
            </div>
            <div v-if="logs.length === 0" class="log-placeholder">
              Awaiting data from container socket...
            </div>
          </div>
        </div>

        <div class="player-sidebar">
          <div class="sticky-sidebar-content">
            <span class="section-label">Connected Users ({{ activeInstance?.players?.length || 0 }})</span>
            <div class="player-list">
              <div
                v-for="player in activeInstance?.players"
                :key="player.uuid"
                class="player-row"
                @click="selectedPlayer = player"
              >
                <div class="player-avatar">{{ player.name.charAt(0).toUpperCase() }}</div>
                <div class="player-meta">
                  <span class="player-name">{{ player.name }}</span>
                  <span class="player-ping">{{ player.ping }}ms ping</span>
                </div>
              </div>

              <div v-if="!activeInstance?.players?.length" class="empty-state-mini">
                No active sessions detected.
              </div>
            </div>

            <transition name="pop">
              <div v-if="selectedPlayer" class="player-detail-card">
                <div class="detail-header">
                  <h3>{{ selectedPlayer.name }}</h3>
                  <button @click="selectedPlayer = null" class="close-btn">×</button>
                </div>
                <div class="detail-body">
                  <div class="stat-item">
                    <label>Vitality</label>
                    <div class="mini-progress">
                      <div class="fill health" :style="{ width: (selectedPlayer.health * 5) + '%' }"></div>
                    </div>
                  </div>
                  <div class="stat-item">
                    <label>Experience</label>
                    <div class="xp-row">
                      <span class="xp-lv">LVL {{ selectedPlayer.level }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </transition>
          </div>
        </div>
      </div>
    </div>

    <div v-if="currentView === 'import'" class="view-layer import-view-container modal-overlay">
      <transition name="pop">
        <div class="modal-glass import-body">
          <header class="modal-header">
            <h2>Import New Server</h2>
         </header>
          <button @click="closeMenu" class="btn-secondary import-view-back">Back</button>
          <div class="import-pane">
            <div class="drop-zone"

            :class="{ 'is-dragging': isDragging }"
            @dragover.prevent="isDragging = true"
            @dragleave.prevent="isDragging = false"
            @drop.prevent="handleFileDrop"
            >
             <div class="drop-content">
               <i class="upload-icon">📄</i>
               <p v-if="!selectedFile">Drag server file here or <span @click="browseFiles" >browse</span></p>
               <p v-else><strong>Selected:</strong> {{ selectedFile.name }}</p>
             </div> 
            </div>

            <div class="form-group input-stack">
              <label>Server Alias</label>
              <input v-model="newInstanceName" class="fancy-input">
            </div>


            <div class="form-row">
            <div class="input-stack half">
              <label>Engine Version</label>
              <select v-model="newInstanceVersion" class="fancy-input">
                <option v-for="v in availableVersions" :key="v" :value="v">{{ v }}</option>
              </select>
            <div class="input-stack">
              <label>Server Software</label>
              <div class="type-grid">
                <div
                  v-for="type in serverTypes"
                  :key="type.id"
                  class="type-box"
                  :class="{ active: selectedType === type.id }"
                  @click="selectedType = type.id"
                >
                  <div class="type-logo">{{ type.logo }}</div>
                  <div class="type-info">
                    <span class="type-name">{{ type.name }}</span>
                    <span class="type-desc">{{ type.desc }}</span>
                  </div>
                </div>
            </div>
            <div class="toggle-stack" @click="isOnlineMode = !isOnlineMode">
              <div class="toggle-info">
                <span class="toggle-label">Online Mode</span>
                <span class="toggle-desc">Require official Mojang authentication</span>
                </div>
                <div class="toggle-switch" :class="{ on: isOnlineMode }"></div>
                </div>
              </div>
            </div>
          </div>
          </div>
          <button class='primary-btn' @click='triggerImport()'>Import Instance</button>
        </div>
      </transition>
    </div>

  </div>
</template>



<style scoped>

  .import-view-back{
    margin: 10px;
  }

  .import-view-container btn-secondary{
    display: flex;
    align-self: left;
  }


  .import-view-container primary-btn{
    margin: 10px;
  }

  .form-group{
    padding: 20px 0px 20px 0px;
    display: flex;
  }

  .form-group label{
    color: rgba(255, 255, 255, 0.9);
    padding: 10px ;
  }

  .import-body h2{
    color: rgba(240, 240, 240, 0.9);
  }


.drop-zone span{
  color: #409eff;
  text-decoration: underline;
  cursor: pointer;
  font-weight: 600;
}

.drop-zone{
  border: 2px dashed rgba(255, 255, 255, 0.3);
  border-radius: 12px;
  padding: 8px;
  text-align: center;
  transition: all 0.3s ease;
  position: relative;
  background: rgba(255, 255, 255, 0.05);
}


drop-zone.is-dragging{
  background: rgba(64,158,255, 0.05);
}


.dashboard-container { padding: 40px; max-width: 1400px; margin: 0 auto; color: #1d1d1f; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; min-height: 100vh; }
.dashboard-header { display: flex; justify-content: space-between; align-items: flex-end; margin-bottom: 40px; flex-shrink: 0; }
h1 { font-size: 2.5rem; font-weight: 900; letter-spacing: -1.5px; margin: 0; }
.title-section p { color: #86868b; margin-top: 4px; }
.title-section span { color: #0071e3; font-weight: 700; }
.active-node { color: #32d74b !important; }

.server-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(360px, 1fr)); gap: 24px; }

.context-container { position: relative; }
.btn-dots {
  background: none; border: none; font-size: 1.2rem; color: #86868b;
  cursor: pointer; padding: 5px 10px; border-radius: 50%; transition: 0.2s;
}
.btn-dots:hover { background: #f5f5f7; color: #1d1d1f; }
.dropdown-menu {
  position: absolute; top: 100%; right: 0; background: white;
  border-radius: 16px; box-shadow: 0 10px 30px rgba(0,0,0,0.1);
  border: 1px solid rgba(0,0,0,0.05); z-index: 50; width: 180px; overflow: hidden; margin-top: 8px;
}
.menu-item {
  width: 100%; border: none; background: none; padding: 12px 16px;
  text-align: left; font-weight: 700; font-size: 0.85rem; cursor: pointer;
}
.menu-item:hover { background: #f5f5f7; }
.menu-item.delete { color: #ff3b30; }
.menu-item.delete:hover { background: #fff1f0; }

.card {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(25px) saturate(160%);
  -webkit-backdrop-filter: blur(25px) saturate(160%);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-top: 1px solid rgba(255, 255, 255, 0.4);
  border-left: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 30px;
  padding: 30px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3), inset 0 0 15px rgba(255, 255, 255, 0.05);
  transition: all 0.4s cubic-bezier(0.165, 0.84, 0.44, 1);
  cursor: pointer;
  overflow: hidden;
  position: relative;
}
.card:hover { transform: translateY(-8px); box-shadow: 0 30px 60px rgba(0,0,0,0.06); }

.card-head { display: flex; justify-content: space-between; align-items: center; position: relative; }
.status-pill {
  display: flex; align-items: center; gap: 8px; font-size: 0.75rem; font-weight: 800;
  text-transform: uppercase; background: #f5f5f7; padding: 6px 14px; border-radius: 20px;
}
.online .status-pill, .running .status-pill { color: #166534; background: #dcfce7; }
.offline .status-pill, .exited .status-pill { color: #1d1d1f; background: #f5f5f7; }

.dot { width: 8px; height: 8px; border-radius: 50%; background: #86868b; }
.online .dot, .running .dot { background: #22c55e; box-shadow: 0 0 10px #22c55e; }

.badge-group { display: flex; gap: 8px; }
.version-badge, .type-badge { font-size: 0.65rem; font-weight: 800; color: #86868b; background: #f5f5f7; padding: 4px 10px; border-radius: 8px; }

.server-info h3 { font-size: 1.4rem; font-weight: 800; margin: 20px 0 5px; }
.server-info code { color: #0071e3; font-weight: 600; font-size: 0.85rem; }

.metrics { margin: 25px 0; }
.metric-labels { display: flex; justify-content: space-between; font-size: 0.7rem; font-weight: 700; color: #86868b; margin-bottom: 8px; }

.progress-bg { height: 6px; background: #f5f5f7; border-radius: 10px; overflow: hidden; }
.progress-fill { height: 100%; background: #0071e3; transition: width 0.8s ease; }

.card-actions { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-top: 25px; }
.btn-action { border: none; padding: 12px; border-radius: 14px; font-weight: 800; cursor: pointer; }
.btn-action.exited, .btn-action.offline { background: #0071e3; color: white; }
.btn-action.running, .btn-action.online { background: #ff3b30; color: white; }
.btn-secondary { background: #f5f5f7; border: none; font-weight: 700; border-radius: 14px; cursor: pointer; padding: 12px; }

.create-card { display: flex; flex-direction: column; align-items: center; justify-content: center; border: 2px dashed #d2d2d7; background: transparent; }
.plus-icon { font-size: 2rem; color: #d2d2d7; margin-bottom: 10px; }
.create-text { text-align: center; }
.create-text h3 { margin: 0; font-size: 1.1rem; }
.create-text p { font-size: 0.8rem; color: #86868b; }

.modal-overlay {
  position: fixed; inset: 0; background: rgba(16, 16, 16, 0.38); backdrop-filter: blur(20px);
  display: flex; align-items: center; justify-content: center; z-index: 1000;
}

.deploy-instance{ color: #ffffff; }

.modal-glass {
  background: rgba(25, 28, 32, 0.51); padding: 40px; border-radius: 40px; width: 100%; max-width: 600px;
  box-shadow: 0 40px 100px rgba(0,0,0,0.1);
}
.input-stack { display: flex; flex-direction: column; gap: 8px; margin: 20px 0; }
.input-stack label { font-size: 0.7rem; font-weight: 800; color: #e5e5e8; text-transform: uppercase; letter-spacing: 0.5px; }
.fancy-input {
  background: rgba(7, 6, 6, 0.18); border: none; padding: 15px; border-radius: 16px;
  font-size: 1rem; font-weight: 600; color: #fffffc; outline: none;
}
.form-row { display: flex; gap: 15px; }
.form-row .half { flex: 1; }

.type-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 12px; margin-top: 5px; }
.type-box {
  display: flex; align-items: center; gap: 12px; padding: 12px; background: rgba(9, 8, 8, 0.17);
  border-radius: 18px; border: 2px solid transparent; cursor: pointer; transition: 0.2s;
}
.type-box.active { border-color: #0071e3; background: rgba(16, 15, 15, 0.17); box-shadow: 0 10px 20px rgba(0,113,227,0.08); }
.type-logo { font-size: 1.5rem; }
.type-info { display: flex; flex-direction: column; }
.type-name { font-weight: 800; font-size: 0.9rem; color: #fffcfc; }
.type-desc { font-size: 0.7rem; color: #86868b; font-weight: 500; }

.toggle-stack {
  display: flex; justify-content: space-between; align-items: center;
  background: rgba(23, 22, 22, 0.16); padding: 15px 20px; border-radius: 20px; cursor: pointer; margin-top: 10px;
}
.toggle-info { display: flex; flex-direction: column; }
.toggle-label { font-size: 0.9rem; font-weight: 700; }
.toggle-desc { font-size: 0.7rem; color: #86868b; }
.toggle-switch {
  width: 44px; height: 24px; background: #d2d2d7; border-radius: 20px; position: relative; transition: 0.3s;
}
.toggle-switch::after {
  content: ''; position: absolute; top: 2px; left: 2px; width: 20px; height: 20px;
  background: white; border-radius: 50%; transition: 0.3s;
}
.toggle-switch.on { background: #32d74b; }
.toggle-switch.on::after { transform: translateX(20px); }

.modal-actions { display: flex; gap: 12px; justify-content: flex-end; margin-top: 30px; }

.stats-view-container { display: flex; flex-direction: column; height: 100%; }
.stats-layout { display: grid; grid-template-columns: 1fr 320px; gap: 30px; flex-grow: 1; min-height: 0; }
.terminal-column { display: flex; flex-direction: column; gap: 12px; }

.log-terminal {
  background: #1d1d1f; border-radius: 24px; padding: 25px;
  height: calc(100vh - 250px);
  overflow-y: auto; overflow-x: hidden;
  color: #32d74b; font-family: 'JetBrains Mono', monospace; font-size: 0.85rem;
  border: 1px solid #30363d; box-shadow: inset 0 0 20px rgba(0,0,0,0.5);
  flex-grow: 1;
}
.log-line { display: flex; gap: 15px; margin-bottom: 4px; opacity: 0.9; word-break: break-all; white-space: pre-wrap; }
.line-number { color: #484f58; min-width: 35px; text-align: right; user-select: none; }
.log-placeholder { color: #484f58; text-align: center; margin-top: 100px; font-style: italic; }

.player-sidebar { position: relative; }
.sticky-sidebar-content { position: sticky; top: 20px; display: flex; flex-direction: column; gap: 20px; }

.section-label {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  color: rgba(255, 255, 255, 0.5);
  padding-left: 10px;
}

.player-list { display: flex; flex-direction: column; gap: 12px; max-height: 400px; overflow-y: auto; padding-right: 5px; }
.player-row {
  display: flex; align-items: center; gap: 15px; padding: 15px; background: white;
  border-radius: 20px; cursor: pointer; border: 1px solid transparent; transition: 0.2s;
}
.player-row:hover { border-color: #0071e3; transform: translateX(5px); }
.player-avatar { width: 36px; height: 36px; background: #0071e3; color: white; border-radius: 10px; display: flex; align-items: center; justify-content: center; font-weight: 900; }
.player-name { display: block; font-weight: 700; font-size: 0.9rem; }
.player-ping { font-size: 0.7rem; color: #86868b; }

.player-detail-card {
  background: white; padding: 20px; border-radius: 24px;
  box-shadow: 0 20px 40px rgba(0,0,0,0.05); border: 1px solid #f5f5f7;
}
.detail-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px; }
.detail-header h3 { margin: 0; font-size: 1.1rem; }
.close-btn { background: none; border: none; font-size: 1.5rem; cursor: pointer; color: #86868b; }
.mini-progress { height: 8px; background: #f5f5f7; border-radius: 10px; overflow: hidden; margin-top: 5px; }
.fill.health { background: #ff3b30; height: 100%; }
.xp-row { margin-top: 5px; }
.xp-lv { font-weight: 800; color: #32d74b; font-size: 0.8rem; }

.pop-enter-active { transition: all 0.3s cubic-bezier(0.165, 0.84, 0.44, 1); }
.pop-enter-from { opacity: 0; transform: scale(0.95); }
.status-msg { padding: 60px; text-align: center; color: #86868b; font-weight: 700; }
.primary-btn { background: #0071e3; color: white; border: none; padding: 14px 24px; border-radius: 18px; font-weight: 700; cursor: pointer; }
.primary-btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>