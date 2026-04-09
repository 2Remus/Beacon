import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'

export default defineConfig(({ command }) => {
  return {
    base: './',
    plugins: [
      vue(),
      // Only enable devtools in development mode
      command === 'serve' ? vueDevTools() : [],
    ],
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url))
      },
    },
    // This entire block now only applies during 'npm run dev'
    server: {
      host: '127.0.0.1', // <--- Force IPv4
      port: 5173,
      strictPort: true,
    },
  }
})