const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electron', {
    node: () => process.versions.node,
    chrome: () => process.versions.chrome,
    // Use arrow functions so they only run when Vue calls them
    getServers: () => ipcRenderer.invoke('getServers'),
    startCloudflare: (port) => ipcRenderer.invoke('startCloudflared', port),
    stopCloudflare: () => ipcRenderer.invoke('stopCloudflared'),
    createServer: (id, name, provider, version, ram, port, online) =>
      ipcRenderer.invoke('createServer', { id, name, provider, version, ram, port, online }),

    startServer: (payload) => ipcRenderer.invoke('startServer',payload)
});