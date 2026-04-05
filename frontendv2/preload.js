import { contextBridge, ipcRenderer } from "electron";


contextBridge.exposeInMainWorld('electron',{
    getServers: ipcRenderer.invoke('getServers')
})