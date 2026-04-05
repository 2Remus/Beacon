import { app, BrowserWindow, ipcMain } from 'electron';
import * as path from 'path';
import { fileURLToPath } from 'url';
import { createRequire } from 'module';

// 1. Manually define __dirname for ES Module scope
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// 2. Create a 'require' function for your Rust binary
const requireNative = createRequire(import.meta.url);

let mainWindow: BrowserWindow | null = null
function createWindow() {
  mainWindow = new BrowserWindow({
    titleBarStyle: 'hidden',
    width: 1200,
    height: 800,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
    }
  })

  mainWindow.loadURL('http://localhost:5173')
}


app.whenReady().then(() => {

  ipcMain.handle('getServers', async() => {
    const  answer = await rust.getServers()
  })
  createWindow()
})