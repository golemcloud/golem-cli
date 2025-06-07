/* eslint-disable @typescript-eslint/no-explicit-any */
import { toast } from "@/hooks/use-toast";
import { fetchData } from "@/lib/tauri&web.ts";
import { ENDPOINT } from "@/service/endpoints.ts";
import { parseErrorResponse } from "@/service/error-handler.ts";
import { Api } from "@/types/api.ts";
import { Component, ComponentList } from "@/types/component.ts";
import { Plugin } from "@/types/plugin";
import { invoke } from "@tauri-apps/api/core";
import { settingsService } from "@/lib/settings.ts";

export class Service {
  public baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  public updateBackendEndpoint = async (url: string) => {
    // await updateIP(url);
    this.baseUrl = url;
  };

  /**
   * getComponents: Get the list of all components
   * Note: Sample Endpoint https://release.api.golem.cloud/v1/components
   * @returns {Promise<Component[]>}
   */
  public checkHealth = async () => {
    const r = await this.callApi("/healthcheck");
    return r;
  };

  /**
   * getComponents: Get the list of all components
   * Note: Sample Endpoint https://release.api.golem.cloud/v1/components
   * @returns {Promise<Component[]>}
   */
  public getComponents = async (appId: string): Promise<Component[]> => {
    const r = await this.callCLI(appId, "component", ["list"]);
    return r as Component[];
  };

  public getComponentById = async (appId: string, componentId: string) => {
    const r = (await this.callCLI(appId, "component", ["list"])) as Component[];
    return r.find(c => c.componentId === componentId);
  };

  public getComponentByIdAndVersion = async (
    appId: string,
    componentId: string,
    version: number,
  ) => {
    const r = (await this.callCLI(appId, "component", ["get"])) as Component[];
    return r.find(c => c.componentId === componentId && c.version === version);
  };

  public createComponent = async (
    appId: string,
    name: string,
    template: string,
  ) => {
    try {
      await this.callCLI(appId, "component", ["new", template, name]);
    } catch (error) {
      console.error("Error in createComponent:", error);
      parseErrorResponse(error);
    }
  };

  public getComponentByName = async (appId: string, name: string) => {
    const r = (await this.callCLI(appId, "component", [
      "get",
      name,
    ])) as Component[];
    return r as Component;
  };

  public updateComponent = async (componenetId: string, form: FormData) => {
    //
  };

  public deletePluginToComponent = async (
    id: string,
    installation_id: string,
  ) => {
    return await this.callApi(
      ENDPOINT.deletePluginToComponent(id, installation_id),
      "DELETE",
    );
  };

  public addPluginToComponent = async (id: string, form: any) => {
    return await this.callApi(
      ENDPOINT.addPluginToComponent(id),
      "POST",
      JSON.stringify(form),
    );
  };

  public upgradeWorker = async (
    appId:string,
    componentName: string,
    workerName: string,
    version: number,
    upgradeType: string,
  ) => {

    return await this.callCLI(appId, "worker", [
      "update",
      `${componentName}/${workerName}`,
      upgradeType,
      `${version}`,
    ]);
  };

  public findWorker = async (
    appId: string,
    componentId: string,
    param = { count: 100, precise: true },
  ) => {
    const component = (await this.getComponentById(
      appId,
      componentId,
    )) as Component;
    const params = [
      "list",
      component.componentName!,
      `--max-count=${param.count}`,
    ];
    if (param.precise) {
      params.push(`--precise`);
    }
    return await this.callCLI(appId, "worker", params);
  };

  public deleteWorker = async (
    appId: string,
    componentId: string,
    workerName: string,
  ) => {
    let component = await this.getComponentById(appId,componentId);
    return await this.callCLI(appId, "worker", [
      "delete",
      `${component?.componentName}/${workerName}`,
    ]);
  };

  public createWorker = async (
    appId: string,
    componentID: string,
    name: string,
  ) => {
    const component = await this.getComponentById(appId, componentID);
    return await this.callCLI(appId, "worker", [
      "new",
      `${component?.componentName!}/${name}`,
      // JSON.stringify(params),
    ]);
  };

  public getApiList = async (appId: string): Promise<Api[]> => {
    // should use yaml
    const r = await this.callCLI(appId, "api", ["definition", "list"]);
    return r as Api[];
  };

  public getApi = async (id: string): Promise<Api[]> => {
    // should use yaml
    // const r = await this.callApi(ENDPOINT.getApi(id));
    // return r as Api[];
  };

  public createApi = async (payload: Api) => {
    // should use yaml
    // const r = await this.callApi(
    //   ENDPOINT.createApi(),
    //   "POST",
    //   JSON.stringify(payload),
    // );
    // return r;
  };

  public deleteApi = async (appId: string, id: string, version: string) => {
    const r = await this.callCLI(appId, "api", [
      "definition",
      "delete",
      `--id=${id}`,
      `--version=${version}`,
    ]);
    return r;
  };

  public putApi = async (id: string, version: string, payload: Api) => {
    // should use yaml
    // const r = await this.callApi(
    //   ENDPOINT.putApi(id, version),
    //   "PUT",
    //   JSON.stringify(payload),
    // );
    // return r;
  };

  public postApi = async (payload: Api) => {
    // const r = await this.callApi(
    //   ENDPOINT.postApi(),
    //   "POST",
    //   JSON.stringify(payload),
    // );
    // return r;
  };

  public getParticularWorker = async (
    appId: string,
    componentId: string,
    workerName: string,
  ) => {
    const component = await this.getComponentById(appId, componentId);
    const r = await this.callCLI(appId, "worker", [
      "get",
      `${component?.componentName}/${workerName}`,
    ]);
    return r;
  };

  public interruptWorker = async (appId: string, workerName: string) => {
    const r = await this.callCLI(appId, "worker", ["interrupt", workerName]);
    return r;
  };

  public resumeWorker = async (appId: string, workerName: string) => {
    const r = await this.callCLI(appId, "worker", ["resume", workerName]);
    return r;
  };

  public invokeWorkerAwait = async (
    appId: string,
    workerName: string,
    functionName: string,
    payload: any,
  ) => {
    const r = await this.callCLI(appId, "worker", [
      "invoke",
      workerName,
      functionName,
      JSON.stringify(payload),
    ]);
    return r;
  };

  public invokeEphemeralAwait = async (
    appId: string,
    functionName: string,
    payload: any,
  ) => {
    const r = await this.callCLI(appId, "worker", [
      "invoke",
      "-",
      functionName,
      JSON.stringify(payload),
    ]);
    return r;
  };

  public getDeploymentApi = async (appId, subdomain: string) => {
    const r = await this.callCLI(appId, "api", [
      "deployment",
      "get",
      subdomain,
    ]);
    return r;
  };

  public deleteDeployment = async (appId: string, subdomain: string) => {
    const r = await this.callCLI(appId, "api", [
      "deployment",
      "delete",
      subdomain,
    ]);
    return r;
  };

  public createDeployment = async (appId: string, subdomain?: string) => {
    const params = ["deployment", "deploy"];
    if (subdomain) {
      params.push(subdomain);
    }
    const r = await this.callCLI(appId, "api", params);
    return r;
  };

  public getOplog = async (
    appId: string,
    componentId: string,
    workerName: string,
    count: number,
    searchQuery: string,
  ) => {
    const component = await this.getComponentById(appId, componentId);
    const r = await this.callCLI(appId, "worker", [
      "oplog",
      `${component?.componentName}/${workerName}`,
      // `--count=${count}`,
      `--query=${searchQuery}`,
    ]);
    console.log(r);

    return r;
  };

  public getComponentByIdAsKey = async (
    appId: string,
  ): Promise<Record<string, ComponentList>> => {
    // Assume getComponents returns a Promise<RawComponent[]>
    const components = await this.getComponents(appId);

    return components.reduce<Record<string, ComponentList>>(
      (acc, component) => {
        const { componentName, componentId, componentType, componentVersion } =
          component;

        // Use componentId as the key. If not available, you might want to skip or handle differently.
        const key = componentId || "";

        // Initialize the component entry if it doesn't exist
        if (!acc[key]) {
          acc[key] = {
            componentName: componentName || "",
            componentId: componentId || "",
            componentType: componentType || "",
            versions: [],
            versionList: [],
          };
        }
        if (acc[key].versionList) {
          acc[key].versionList.push(componentVersion);
        }
        if (acc[key].versions) {
          acc[key].versions.push(component);
        }
        return acc;
      },
      {},
    );
  };

  public getPlugins = async (appId: string): Promise<Plugin[]> => {
    return await this.callCLI(appId, "plugin", ["list"]);
  };

  public getPluginByName = async (
    appId: string,
    name: string,
    version: string,
  ): Promise<Plugin[]> => {
    return await this.callCLI(appId, "plugin", ["get", name, version]);
  };

  // public downloadComponent = async (
  //   componentId: string,
  //   version: number,
  // ): Promise<any> => {
  //   return await this.downloadApi(
  //     ENDPOINT.downloadComponent(componentId, version),
  //   );
  // };
  public createPlugin = async (appId: string, manifestFileLocation: string) => {
    return await this.callCLI(appId, "plugin", [
      "register",
      manifestFileLocation,
    ]);
  };
  public deletePlugin = async (
    appId: string,
    name: string,
    version: string,
  ) => {
    return await this.callCLI(appId, "plugin", ["unregister", name, version]);
  };

  private callCLI = async (
    appId: string,
    command: string,
    subcommands: string[],
  ): Promise<any> => {
    // find folder location
    const app = await settingsService.getAppById(appId);
    if (!app) {
      throw new Error("App not found");
    }
    //  we use invoke here to call a special command that calls golem CLI for us
    let result: string;
    try {
      result = await invoke("call_golem_command", {
        command,
        subcommands,
        folderPath: app.folderLocation,
      });
    } catch (e) {
      toast({
        title: "Error in calling golem CLI",
        description: String(e),
        variant: "destructive",
        duration: 5000,
      });
      throw new Error("Error in calling golem CLI")
    }

    let parsedResult;
    const match = result.match(/(\[.*\]|\{.*\})/s);
    if (match) {
      try {
        parsedResult = JSON.parse(match[0]);
      } catch (e) {
        // some actions do not return json
      }
    }
    return parsedResult || true;
  };

  // private downloadApi = async (
  //   url: string,
  //   method: string = "GET",
  //   data: FormData | string | null = null,
  //   headers = { "Content-Type": "application/json" },
  // ): Promise<any> => {
  //   const resp = await fetchData(`${this.baseUrl}${url}`, {
  //     method: method,
  //     body: data,
  //     headers: headers,
  //   })
  //     .then(res => {
  //       if (res.ok) {
  //         return res;
  //       }
  //     })
  //     .catch(err => {
  //       toast({
  //         title: "Api is Failed check the api details",
  //         variant: "destructive",
  //         duration: 5000,
  //       });
  //       throw err;
  //     });
  //   return resp;
  // };
}
