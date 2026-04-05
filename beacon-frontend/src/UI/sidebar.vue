<template>
  <aside class="sidebar">
    <nav class="nav">
      <router-link v-for="item in menu" :key="item.name" :to="item.path" class="nav-item">
        <span class="nav-icon">{{ item.icon }}</span>
        <span class="nav-label">{{ item.name }}</span>
      </router-link>
    </nav>
  </aside>
</template>

<script setup>
const menu = [
  { name: 'Server', path: '/', icon: '☰' },
  // { name: 'Options', path: '/settings', icon: '⚙' },
  // { name: 'Console', path: '/console', icon: '💻' },
  { name: 'Connections', path: '/connections', icon: '🌐' },
  { name: 'Backend', path: '/backend', icon: '📁' }
]
</script>

<style scoped>
.sidebar {
  width: 220px;
  background: #1e2124;
  padding: 10px 0;
}

.nav-item {
  display: flex;
  align-items: center;
  padding: 12px 20px;
  text-decoration: none;
  color: #adb5bd;
  transition: 0.2s;
  border-left: 4px solid transparent;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #fff;
}

.router-link-active {
  background: #2d3136;
  color: #fff;
  border-left-color: #00abf0;
}

.nav-icon { margin-right: 15px; font-size: 1.1rem; }
.nav-label { font-size: 0.95rem; font-weight: 600; }
</style>