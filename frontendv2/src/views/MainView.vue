<script setup>
import Sidebar from "@/UI/sidebar.vue";
</script>
<template>
  <div class="aternos-shell">
    <header class="top-bar">
      <div class="logo">BEACON</div>
      <div class="user-menu">
        <span class="username">Admin</span>
        <div class="avatar-sm"></div>
      </div>
    </header>

    <div class="main-container">
      <Sidebar />
      <main class="content-view">
        <div class="page-wrapper">
          <router-view />
        </div>
      </main>
    </div>
  </div>
</template>

<style scoped>
.aternos-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: #2d3136; /* Aternos Dark Grey */
  color: #fff;
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
}

.top-bar {
  height: 60px;
  background: #1e2124;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 20px;
  border-bottom: 1px solid #141619;
}

.logo {
  font-weight: 900;
  font-size: 1.4rem;
  color: #00abf0; /* Aternos Blue */
  letter-spacing: 1px;
}

.main-container {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.content-view {
  flex: 1;
  background: #2d3136;
  overflow-y: auto;
  padding: 20px;
}

.page-wrapper {
  max-width: 1100px;
  margin: 0 auto;
}
</style>