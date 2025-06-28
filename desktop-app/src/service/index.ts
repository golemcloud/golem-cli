import { Service } from "@/service/client.ts";
// @ts-nocheck
// import { fetchCurrentIP } from "@/lib/tauri&web.ts";

export let API: Service = new Service("http://localhost:9881");

(async () => {
  API = new Service("http://localhost:9881");
})();

export async function updateService(url: string) {
  if (API) {
    console.log(url)
    // await API.updateBackendEndpoint(url);
  }
}

export async function getEndpoint() {
  return "http://localhost:9881"
}