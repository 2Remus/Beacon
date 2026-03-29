// TOP of electron-main.cjs
const util = require('util');
const fs = require('fs');
const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const { spawn, exec } = require('child_process');
const sudo = require('sudo-prompt');

// Polyfill for sudo-prompt compatibility with modern Node
if (!util.isObject) {
    util.isObject = (obj) => obj !== null && typeof obj === 'object';
}
if (!util.isFunction) {
    util.isFunction = (fn) => typeof fn === 'function';
}
if (!util.isString) {
    util.isString = (str) => typeof str === 'string';
}

let hostProcess, apiProcess;
const isDev = !app.isPackaged;
let mainWindow;
let splash;

// --- DYNAMIC PATHING ---
const resourcesPath = isDev ? process.cwd() : process.resourcesPath;
const binPath = path.join(resourcesPath, 'bin');

/**
 * GATEKEEPER: Checks if beacon.local exists natively.
 * This prevents the sudo prompt if the hosts are already configured.
 */
function updateSystemHosts(win) {
    const beaconDomain = "beacon.local";
    const hostsPath = process.platform === 'win32'
        ? 'C:\\Windows\\System32\\drivers\\etc\\hosts'
        : '/etc/hosts';

    try {
        if (fs.existsSync(hostsPath)) {
            const content = fs.readFileSync(hostsPath, 'utf8');
            // Use word boundary to avoid partial matches
            const hasEntry = new RegExp(`\\b${beaconDomain}\\b`, 'i').test(content);

            if (hasEntry) {
                console.log(`[Beacon] ${beaconDomain} verified. Skipping password prompt.`);
                return startSystems(win);
            }
        }
    } catch (e) {
        console.error("[Beacon] Read error on hosts:", e.message);
    }

    // If not found, proceed with elevated write
    console.log(`[Beacon] ${beaconDomain} missing. Requesting elevation...`);
    const options = { name: 'Beacon Hub' };
    const entry = "127.0.0.1 beacon.local app.beacon.local api.beacon.local sso.beacon.local";

    const writeCmd = process.platform === 'win32'
        ? `cmd /c "echo ${entry} >> ${hostsPath}"`
        : `sh -c 'echo "${entry}" >> ${hostsPath}'`;

    sudo.exec(writeCmd, options, (sudoErr) => {
        if (sudoErr) console.error('[Beacon] Sudo Failed:', sudoErr);
        else console.log("[Beacon] Hosts updated successfully.");
        startSystems(win);
    });
}

/**
 * PROCESS MANAGEMENT: Starts the Beacon Host (Docker/Infra)
 */
function startSystems(mainWindow) {
    const isWin = process.platform === 'win32';
    const binaryExt = isWin ? '.exe' : '';
    const hostBinary = path.join(binPath, `beacon-host${binaryExt}`);

    hostProcess = spawn(hostBinary, [], {
        shell: true,
        cwd: resourcesPath,
        env: {
            ...process.env,
            BEACON_ROOT: resourcesPath,
            // Ensure common paths are available for Docker/System commands
            PATH: process.env.PATH + (isWin ? '' : ':/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin')
        }
    });

    hostProcess.stdout.on('data', (data) => {
        const line = data.toString();
        console.log(`[Host]: ${line}`);

        if (line.includes("READY: DOCKER_STACK_UP")) {
            console.log("[Beacon] Infrastructure Ready. Booting API...");
            startApi(mainWindow, binPath, binaryExt);
        }
    });
}

/**
 * API MANAGEMENT: Starts the Rust Backend
 */
function startApi(mainWindow, binPath, binaryExt) {
    const apiBinary = path.join(binPath, `beacon-api${binaryExt}`);

    apiProcess = spawn(apiBinary, [], {
        cwd: resourcesPath,
        shell: true,
        env: {
            ...process.env,
            BEACON_ROOT: resourcesPath,
            DATABASE_URL: "postgresql://user:password@127.0.0.1:5436/beacon"
        }
    });

    apiProcess.stdout.on('data', (data) => {
        const line = data.toString();
        console.log(`[API]: ${line}`);

        if (line.includes("BEACON_API_LIVE") || line.includes("Listening on")) {
            console.log("[Beacon] API Live. Transitioning to Dashboard...");

            mainWindow.webContents.reload();

            setTimeout(() => {
                if (splash && !splash.isDestroyed()) splash.close();
                mainWindow.show();
                mainWindow.focus();
            }, 200);
        }
    });
}

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
        width: 1280,
        height: 800,
        show: false,
        titleBarStyle: 'hidden',
        backgroundColor: '#1e2126', // Matches your "Cloud Instances" theme
        webPreferences: {
            preload: path.join(__dirname, 'preload.js'),
            contextIsolation: true
        }
    });

    mainWindow.loadURL('http://127.0.0.1:5173');

    // Entry point: Verify hosts then start services
    updateSystemHosts(mainWindow);
}

app.whenReady().then(createWindow);

// --- CLEANUP ---
app.on('will-quit', () => {
    // SIGINT allows Rust to run its internal cleanup (just ensure that update_hosts(..., false) doesn't wipe your file!)
    if (hostProcess) hostProcess.kill('SIGINT');
    if (apiProcess) apiProcess.kill('SIGINT');
});

app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') app.quit();
});