<script setup>
import { RouterView } from 'vue-router';



</script>

<style scoped>
  /* Match the main content background */
  .titlebar {
    background: #1e1f22; /* Keep it dark like Discord's sidebars */
    height: 40px;
    display: flex;
    align-items: center;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
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
  <RouterView/>
</template>

