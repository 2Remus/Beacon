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
  { name: 'Connections', path: '/connections', icon: '🌐' },
  { name: 'Connect', path: '/connect', icon: '💻' },
  //{ name: 'Backend', path: '/backend', icon: '📁' }
]
</script>

<style scoped>
.sidebar {
  width: 240px;
  /* Liquid Glass Base */
  background: rgba(25, 28, 32, 0.51);
  backdrop-filter: blur(20px) saturate(160%);
  -webkit-backdrop-filter: blur(20px) saturate(160%);
  padding: 20px 0;
  /* Thin glass edge on the right */
  border-right: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  flex-direction: column;
  border-radius: 10px;
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 0 12px;
}

.nav-item {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  text-decoration: none;
  color: rgba(255, 255, 255, 0.5);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  border-radius: 12px;
  border: 1px solid transparent;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #fff;
  transform: translateX(4px);
}

/* Active State: Illuminating the Glass */
.router-link-active {
  background: rgba(0, 171, 240, 0.1);
  color: #00abf0;
  border: 1px solid rgba(0, 171, 240, 0.2);
  box-shadow:
    0 4px 15px rgba(0, 0, 0, 0.1),
    inset 0 0 10px rgba(0, 171, 240, 0.05);
}

.nav-icon {
  margin-right: 12px;
  font-size: 1.2rem;
  transition: transform 0.3s ease;
}

.router-link-active .nav-icon {
  transform: scale(1.1);
  filter: drop-shadow(0 0 8px rgba(0, 171, 240, 0.5));
}

.nav-label {
  font-size: 0.9rem;
  font-weight: 700;
  letter-spacing: 0.3px;
}

/* Glass Refraction Detail at the bottom */
.sidebar::after {
  content: '';
  margin-top: auto;
  height: 1px;
  width: 80%;
  margin-left: 10%;
  background: linear-gradient(90deg, transparent, rgba(255,255,255,0.1), transparent);
}
</style>