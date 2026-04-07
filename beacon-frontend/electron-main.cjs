// TOP of electron-main.cjs
const util = require('util');
const fs = require('fs');
const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const { spawn, exec } = require('child_process');
const sudo = require('sudo-prompt');
    const rust = require(path.join(__dirname, '../backendv2/index.darwin-arm64.node'));

let hostProcess, apiProcess;
const isDev = !app.isPackaged;
let mainWindow;
let splash;

// --- DYNAMIC PATHING ---
const resourcesPath = isDev ? process.cwd() : process.resourcesPath;






function createWindow() {
    // Splash screen configuration
    splash = new BrowserWindow({
        width: 450,
        height: 550,
        transparent: true,
        frame: false,
        alwaysOnTop: true,
        resizable: false,
        center: true,
        webPreferences: { nodeIntegration: true }
    });
    splash.loadFile('splash.html');

    // Main Dashboard configuration
    mainWindow = new BrowserWindow({
        webPreferences: {
            preload: path.join(__dirname, 'preload.js'),
            contextIsolation: true
        },
        width: 1280,
        height: 800,
        show: false, // Keep this false to prevent the white flash
        titleBarStyle: 'hidden',
        backgroundColor: '#1e2126',

    });

    // LOAD LOGIC
    if (isDev) {
            mainWindow.loadURL('http://127.0.0.1:5173');
    } else {
        // Points to the dist folder created by Vite
        mainWindow.loadFile(path.join(__dirname, 'dist/index.html'));
    }

    // THE FIX: Show window when content is ready
    mainWindow.once('ready-to-show', () => {
        splash.close(); // Close the splash screen
        mainWindow.show(); // Finally show the main window
    });


    mainWindow.loadURL('http://127.0.0.1:5173');

}

app.whenReady().then(() => {
    // This "catches" the 'get-servers' call from Vue
    ipcMain.handle('get-servers', async (event, args) => {
        try {
            // NAPI-RS converts snake_case to camelCase by default
            // So rust.get_servers() becomes rust.getServers()
            return await rust.getServers();
        } catch (err) {
            console.error("Rust execution error:", err);
            throw err; // Sends the error back to Vue's catch block
        }
    });

    ipcMain.handle('start-cloudflared', async (event, arg) => {
        const result = await rust.startCloudflared(25565);
        return result;
    });

    createWindow();
});

// --- CLEANUP ---
app.on('will-quit', () => {
    // SIGINT allows Rust to run its internal cleanup (just ensure that update_hosts(..., false) doesn't wipe your file!)
    if (hostProcess) hostProcess.kill('SIGINT');
    if (apiProcess) apiProcess.kill('SIGINT');
});

app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') app.quit();
});