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
    startServer: (payload) => ipcRenderer.invoke('startServer',payload),
    getLogs: (id) => ipcRenderer.invoke('subscribe-to-logs', id),

    //need to study how this really works
    onLogUpdate: (id, callback) => {
        const channel = `logs:${id}`;

        // Create the listener function
        const listener = (_event, value) => callback(value);

        // Attach the listener to the specific server's channel
        ipcRenderer.on(channel, listener);

        // Returns a cleanup function.
        // In Vue, you can call this in onUnmounted() to prevent memory leaks.
        return () => {
            ipcRenderer.removeListener(channel, listener);
        };

    }, 

    serverImport: (id, name, provider, version, path) => ipcRenderer.invoke('import-server',{id, name, provider, version, path}),
    stopServer: () => ipcRenderer.invoke('kill-container'),
    killContainers: (id) => ipcRenderer.invoke('killContainers', id),
    clientConnect: (url) => ipcRenderer.invoke('tunnel-connect', url),

});