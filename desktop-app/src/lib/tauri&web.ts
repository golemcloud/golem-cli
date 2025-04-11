import { BaseDirectory } from "@tauri-apps/api/path";
import TauriWebSocket from "@tauri-apps/plugin-websocket";
import { invoke } from "@tauri-apps/api/core";
import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
import { writeFile } from "@tauri-apps/plugin-fs";
import { getVersion } from "@tauri-apps/api/app";

async function checkIfTauri() {
  try {
    await getVersion();
    return true;
  } catch (error) {
    return false;
  }
}

async function checkEnvironment() {
  const isTauri = await checkIfTauri();
  const isClientSide = typeof window !== "undefined";

  return {
    isTauri,
    isClientSide,
  };
}

const { isTauri, isClientSide } = await checkEnvironment();

export async function saveFile(fileName: string, data: Uint8Array) {
  if (isTauri) {
    // Use Tauri to save the file in the Downloads directory
    await writeFile(fileName, data, { baseDir: BaseDirectory.Download });
  } else if (isClientSide) {
    // Use Blob and createObjectURL for web downloads
    const blob = new Blob([data]);
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = fileName;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }
}

export async function updateIP(newIP: string) {
  try {
    await invoke("update_backend_ip", { newIp: newIP });
    console.log("Backend IP updated!");
  } catch (error) {
    console.error("Failed to update IP:", error);
  }
}

// Retrieve the backend IP (for example, on app startup)
export async function fetchCurrentIP() {
  try {
    const ip: string = await invoke("get_backend_ip");
    console.log("Current backend IP:", ip);
    return ip;
  } catch (error) {
    console.error("Failed to get current IP:", error);
    return "http://localhost:9881";
  }
}

export async function fetchData(
  url: string,
  options?: RequestInit,
): Promise<Response> {
  if (isClientSide) {
    return window.fetch(url, options);
  }

  if (isTauri) {
    return tauriFetch(url, options); 
  }
  
  return fetch(url, options);
}

export class UniversalWebSocket {
  private ws: WebSocket | TauriWebSocket;

  constructor(ws: WebSocket | TauriWebSocket) {
    this.ws = ws;
  }

  static async connect(url: string): Promise<UniversalWebSocket> {
    if (isTauri) {
      return new UniversalWebSocket(await TauriWebSocket.connect(url));
    }
    return new UniversalWebSocket(new WebSocket(url));
  }

  public send(data: never) {
    const message = JSON.stringify(data);
    if (isTauri) {
      (this.ws as TauriWebSocket)
        .send(message)
        .then(() => {})
        .catch(console.error);
    } else {
      (this.ws as WebSocket).send(message);
    }
  }

  public close() {
    if (isTauri) {
      (this.ws as TauriWebSocket)
        .disconnect()
        .then(() => {})
        .catch(console.error);
    } else {
      (this.ws as WebSocket).close();
    }
  }

  public onMessage(callback: (data: unknown) => void) {
    if (isTauri) {
      (this.ws as TauriWebSocket).addListener(event => {
        try {
          const message = event.data;
          if (typeof message === "string") {
            callback(JSON.parse(message));
          }
        } catch (e) {
          console.error("Failed to parse Tauri WebSocket message", e);
        }
      });
    } else {
      (this.ws as WebSocket).onmessage = event => {
        try {
          callback(JSON.parse(event.data));
        } catch (e) {
          console.error("Failed to parse WebSocket message", e);
        }
      };
    }
  }
}
