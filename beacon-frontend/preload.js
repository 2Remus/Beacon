const { contextBridge, ipcRenderer } = require('electron')

contextBridge.exposeInMainWorld('electron', {
    node: () => process.versions.node,
    chrome: () => process.versions.chrome,
    // This just sends a signal to the Main process
    getServers: () => ipcRenderer.invoke('get-servers'),
    onServerUpdate: (callback) => ipcRenderer.on('server-update', (_event, value) => callback(value)),
    startCloudflare: () => ipcRenderer.invoke('start-cloudflared'),
})