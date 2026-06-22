<script setup>
import { RouterView } from 'vue-router'
</script>

<style scoped>
/* Match the main content background */
html, body {
  margin: 0;
  padding: 0;
  width: 100vw;
  height: 100vh;
  overflow: hidden; /* Prevents unwanted secondary app container scrollbars */
  background-color: #1e1e2e; /* Match this to your exact Vue app theme color to prevent layout flashing */
}

#app {
  width: 100%;
  height: 100%;
  margin: 0;
  padding: 0;
}

.titlebar {
  background: #1e1f22; /* Keep it dark like Discord's sidebars */
  height: 40px;
  display: flex;
  align-items: center;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  color: white;
  font-weight: 900;
  justify-content: center; /* This centers your text! */
  padding-left: 80px; /* Offset for Mac buttons */
  padding-right: 80px; /* Symmetrical padding so center is true center */
  -webkit-app-region: drag;
}

/* Ensure the logo/text has its own style */
.titlebar-text {
  font-size: 14px;
  font-weight: 600;

  color: #949ba4; /* Muted gray for that subtle pro look */
}
</style>

<template>
  <header class="titlebar">
    <div class="drag-region">Beacon Hub</div>
  </header>
  <RouterView />
</template>
