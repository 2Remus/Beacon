import { app, BrowserWindow, ipcMain } from 'electron';
import * as path from 'path';
import { fileURLToPath } from 'url';
import { createRequire } from 'module';

// 1. Manually define __dirname for ES Module scope
// In ESM, __dirname and __filename are not globally available
const __filename: string = fileURLToPath(import.meta.url);
const __dirname: string = path.dirname(__filename);
export const wait = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));
// 2. Create a 'require' function for native modules or Rust binaries
// This is essential if you are calling Rust via N-API or similar
const requireNative = createRequire(import.meta.url);

//const rust = requireNative('./bin/index.darwin-arm64.node');
//console.log('rust binary loaded successfully')


let rust: any;

const isProd = app.isPackaged;

// Helper to get the correct path regardless of dev/prod
const getBinPath = () => {
  if (isProd) {
    // Standard location for extraResources in packaged apps
    return path.join(process.resourcesPath, 'bin', 'index.darwin-arm64.node');
  } else {
    // app.getAppPath() usually points to your project root in dev
    return path.join(app.getAppPath(), 'bin', 'index.darwin-arm64.node');
  }
};


const binPath = getBinPath();

try {
  rust = requireNative(binPath);
  console.log('Rust exports:', Object.keys(rust));
} catch (err) {
  console.error('Core Logic Error: Failed to load Rust binary.');
  console.error('Attempted path:', binPath);
  console.error(err);
}


console.log(binPath);


console.log(rust)


function registerIpcHandlers() {
  // Use .handleOnce if you only need it once, or check if already registered
  ipcMain.handle('startCloudflared', async (event, port) => {
    console.log(`Starting tunnel on port ${port}`);
    try {
      return await rust.startCloudflared(port);
    } catch (e) {
      console.error("Rust error:", e);
      throw e; // This will reject the promise in the frontend
    }
  })


  ipcMain.handle('stopCloudflared', async () => {
      try{
        rust.stopCloudflared();

      }catch (e) {
        console.error("Rust layer error: ", e)
      }
  })

  ipcMain.handle('createServer', async (event, data) => {
    // We extract the fields from 'data' and pass them individually

    const onlineMode = data.online_mode === true || data.online_mode === 'true';


    return await rust.createServer(
      data.id,
      data.name,
      data.provider,
      data.version,
      parseInt(data.ram_mb) || 3072, // Convert "3G" string to a Number
      data.port,
      onlineMode,
    );
  });


  ipcMain.handle('startServer', async (event, payload) => {
    // 1. Destructure the keys coming from your Vue 'payload' object
    const { id, bin_dir, ram } = payload;

    // 2. CRITICAL: Log these to your TERMINAL to see which one is missing
    console.log("--- IPC Debug ---");
    console.log("ID:", id);
    console.log("Path:", bin_dir);
    console.log("RAM:", ram);

    try {
      // 3. Pass them as INDIVIDUAL arguments, not as one object
      // Rust signature: spawn_container(id: String, bin_dir: String, ram: u32)
      return await rust.spawnContainer(
        id.toString(),
        bin_dir.toString(),
        Number(ram)
      );
    } catch (e) {
      console.error("Rust bridge crash:", e);
      return { error: e.toString() };
    }
  });


  ipcMain.handle('getServers', async () => {
    try{
      return await rust.getServers();
    }
    catch (e){
      console.error("Rust error:", e);
    }
  })

  ipcMain.handle('subscribe-to-logs', (event, serverId: string) => {
    // Call your Rust #[napi] function
    // The second argument is the ThreadsafeFunction (callback)
    rust.streamLogs(serverId, (err: any, logLine: string) => {
      if (err) {
        console.error(`Log stream error for ${serverId}:`, err);
        return;
      }

      // Send the line to the frontend via a unique channel for this server
      event.sender.send(`logs:${serverId}`, logLine);
    });
  });


  ipcMain.handle('stopServers', async (event, serverId: string) => {
    try{
      return await rust.killContainers();
    }catch (e){
      console.error("Rust error:", e);
    }
  })


}


let mainWindow: BrowserWindow | null = null;
let splashWindow: BrowserWindow | null = null;

function createWindow(): void {
  splashWindow = new BrowserWindow({
    width: 500,
    height: 300,
    transparent: true, // Makes the "liquid glass" look better if splash.html has rounded corners
    frame: false,      // No window controls
    alwaysOnTop: true,
    webPreferences: {
      contextIsolation: true,
    }
  });

  splashWindow.loadFile(path.join(__dirname, '../splash.html'));
  splashWindow.center();

  mainWindow = new BrowserWindow({
    titleBarStyle: 'hidden',
    show: false,
    width: 1200,
    height: 800,
    webPreferences: {
      // Ensure the preload path points to the compiled .js file
      preload: path.join(__dirname, './preload.js'),
      // Security best practices:
      contextIsolation: true,
      nodeIntegration: false,
    }
  });

  const cleanupResources = () => {
    console.log("Cleaning up Rust containers...");
    try {
      // This calls your N-API function
      const result = rust.killContainers();
      console.log(result);
    } catch (err) {
      console.error("Failed to kill containers:", err);
    }
  };

  app.on('before-quit', () => {
    console.log("Shutting down... calling Rust cleanup.");
    cleanupResources();
  });


  ipcMain.handle('kill-container', async (event, id ) => {
    try{
        const result = rust.killContainers(id);
    }catch (e) {
      console.error("Rust error:", e);
    }
  })


  ipcMain.handle('tunnel-connect', async (event,id) => {
    const result = await rust.clientConnect(id);
  })
  if (!isProd) {
    // Load from the Vite dev server
    mainWindow.loadURL('http://localhost:5173');
    mainWindow.webContents.openDevTools();
  } else {
    // Load the compiled index.html from the dist folder
    // __dirname in production usually points to the 'dist-electron' or 'resources' folder
    mainWindow.loadFile(path.join(__dirname, './dist/index.html'));
  }

  mainWindow.once('ready-to-show', async () => {
    await wait(2000);
    if (splashWindow) splashWindow.close();
    mainWindow?.show();
    mainWindow?.focus();

  });


  mainWindow.on('closed', () => {
    // If this is the only window, quitting will trigger 'before-quit' anyway
    mainWindow = null;
  })
}

// Electron lifecycle management
app.whenReady().then(() => {


  registerIpcHandlers();
  createWindow();

  app.on('activate', () => {

    if (BrowserWindow.getAllWindows().length === 0) {

      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  // On macOS, apps stay open even without windows.
  // If you want containers to die when the window closes, keep this here.
  if (process.platform !== 'darwin') {
    app.quit();
  }
});