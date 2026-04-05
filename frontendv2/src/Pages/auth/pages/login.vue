<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const isLoading = ref(false)
const email = ref('')
const password = ref('')
const error = ref('')

const handleLogin = async () => {
  if (!email.value || !password.value) {
    error.value = 'Please enter your credentials.'
    return
  }

  isLoading.value = true
  error.value = ''

  try {
    // In a real scenario, this links to your Keycloak/Axum backend
    console.log('Authenticating with Beacon Control Plane...')

    // Simulating a successful login delay
    setTimeout(() => {
      isLoading.value = false
      router.push('/dashboard')
    }, 1500)
  } catch (err) {
    isLoading.value = false
    error.value = 'Invalid cluster credentials.'
  }
}
</script>

<template>
  <div class="login-page">
    <div class="blob-1"></div>
    <div class="blob-2"></div>

    <div class="login-card">
      <div class="brand-section">
        <div class="logo-mark"></div>
        <h1>Beacon</h1>
        <p>Control Plane Authentication</p>
      </div>

      <form @submit.prevent="handleLogin" class="login-form">
        <div v-if="error" class="error-pill">{{ error }}</div>

        <div class="input-group">
          <label>Email Address</label>
          <input
              v-model="email"
              type="email"
              placeholder="name@company.com"
              class="glass-input"
              required
          />
        </div>

        <div class="input-group">
          <label>Password</label>
          <input
              v-model="password"
              type="password"
              placeholder="••••••••"
              class="glass-input"
              required
          />
        </div>

        <button type="submit" class="login-btn" :disabled="isLoading">
          <span v-if="!isLoading">Sign In to Cluster</span>
          <span v-else class="loader"></span>
        </button>
      </form>

      <footer class="login-footer">
        <p>SVG-North Region / System Architect Access Only</p>
      </footer>
    </div>
  </div>


</template>

<style scoped>
.login-page {
  height: 100vh;
  width: 100vw;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f5f5f7;
  overflow: hidden;
  position: relative;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
}

/* Glassmorphism Card */
.login-card {
  width: 100%;
  max-width: 420px;
  padding: 50px;
  background: rgba(255, 255, 255, 0.7);
  backdrop-filter: blur(25px);
  border: 1px solid rgba(255, 255, 255, 0.8);
  border-radius: 35px;
  box-shadow: 0 40px 100px rgba(0, 0, 0, 0.08);
  z-index: 10;
  text-align: center;
}

.brand-section { margin-bottom: 40px; }
.logo-mark {
  width: 50px;
  height: 50px;
  background: #0071e3;
  border-radius: 14px;
  margin: 0 auto 20px;
  box-shadow: 0 10px 20px rgba(0, 113, 227, 0.2);
}

h1 { font-size: 2rem; font-weight: 900; margin: 0; color: #1d1d1f; letter-spacing: -1px; }
.brand-section p { color: #86868b; margin-top: 8px; font-size: 0.9rem; }

/* Form Elements */
.login-form { text-align: left; }
.input-group { margin-bottom: 24px; }
label { display: block; font-size: 0.75rem; font-weight: 800; color: #86868b; text-transform: uppercase; margin-bottom: 8px; letter-spacing: 0.5px; }

.glass-input {
  width: 100%;
  padding: 16px;
  background: rgba(245, 245, 247, 0.8);
  border: 1px solid transparent;
  border-radius: 14px;
  font-size: 1rem;
  transition: all 0.2s ease;
}

.glass-input:focus {
  background: #fff;
  border-color: #0071e3;
  box-shadow: 0 0 0 4px rgba(0, 113, 227, 0.1);
  outline: none;
}

.login-btn {
  width: 100%;
  padding: 16px;
  background: #1d1d1f;
  color: white;
  border: none;
  border-radius: 16px;
  font-weight: 700;
  font-size: 1rem;
  cursor: pointer;
  transition: transform 0.2s, background 0.2s;
  margin-top: 10px;
}

.login-btn:hover:not(:disabled) { background: #000; transform: translateY(-2px); }
.login-btn:disabled { opacity: 0.6; cursor: not-allowed; }

.error-pill {
  background: #fee2e2;
  color: #991b1b;
  padding: 12px;
  border-radius: 12px;
  font-size: 0.85rem;
  font-weight: 600;
  margin-bottom: 20px;
  text-align: center;
}

.login-footer { margin-top: 40px; border-top: 1px solid rgba(0,0,0,0.05); padding-top: 20px; }
.login-footer p { font-size: 0.7rem; color: #c1c1c6; font-weight: 700; text-transform: uppercase; }

/* Background Blobs */
.blob-1, .blob-2 {
  position: absolute;
  width: 500px;
  height: 500px;
  background: radial-gradient(circle, rgba(0, 113, 227, 0.08) 0%, transparent 70%);
  border-radius: 50%;
  z-index: 1;
}
.blob-1 { top: -100px; right: -100px; }
.blob-2 { bottom: -100px; left: -100px; }

/* Simple Loader */
.loader {
  width: 20px;
  height: 20px;
  border: 3px solid rgba(255,255,255,0.3);
  border-top-color: #fff;
  border-radius: 50%;
  display: inline-block;
  animation: spin 1s ease-in-out infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>