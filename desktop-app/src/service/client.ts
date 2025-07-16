import { toast } from "@/hooks/use-toast";
// import { fetchData } from "@/lib/tauri&web.ts";
// import { ENDPOINT } from "@/service/endpoints.ts";
import { parseErrorResponse } from "@/service/error-handler.ts";
import { Api } from "@/types/api.ts";
import { Component, ComponentList, Parameter } from "@/types/component.ts";
import { Plugin } from "@/types/plugin";
import { invoke } from "@tauri-apps/api/core";
import { settingsService } from "@/lib/settings.ts";
import {
  exists,
  readDir,
  readTextFile,
  writeTextFile,
} from "@tauri-apps/plugin-fs";
import { join } from "@tauri-apps/api/path";
import {
  GolemApplicationManifest,
  HttpApiDefinition,
  serializeHttpApiDefinition,
} from "@/types/golemManifest.ts";
import { parse, parseDocument, Document, YAMLMap } from "yaml";
import {
  convertToWaveFormatWithType,
  convertValuesToWaveArgs,
} from "@/lib/wave";

export class Service {
  public baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  /**
   * getComponents: Get the list of all components
   * Note: Sample Endpoint https://release.api.golem.cloud/v1/components
   * @returns {Promise<Boolean>}
   */
  public checkHealth = async (): Promise<Boolean> => {
    return Promise.resolve(true);
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
    const c = r.find(c => c.componentId === componentId);
    if (!c) {
      throw new Error("Could not find component");
    }
    return c;
  };

  /**
   * getComponentYamlPath: Get the path to the YAML file of a component
   * @param appId - The ID of the application
   * @param componentName - The name of the component
   * @returns {Promise<string | null>} - The path to the YAML file or null if not found
   */
  public async getComponentYamlPath(
    appId: string,
    componentName: string,
  ): Promise<string> {
    const app = await settingsService.getAppById(appId);
    if (!app) {
      throw new Error("App not found");
    }

    // Replace: with - in component name
    let folderName = componentName.replace(/:/g, "-").toLowerCase();

    try {
      // Get all folders in app.folderLocation
      const appEntries = await readDir(app.folderLocation);
      const appFolders = appEntries
        .filter(entry => entry.isDirectory)
        .map(entry => entry.name);

      // Find all folders starting with "components-"
      const componentsFolders = appFolders.filter(folder =>
        folder.startsWith("components-"),
      );

      // Search through each component-* folder for the component
      for (const componentsFolder of componentsFolders) {
        const componentsFolderPath = await join(
          app.folderLocation,
          componentsFolder,
        );

        try {
          const subEntries = await readDir(componentsFolderPath);
          const subFolders = subEntries
            .filter(entry => entry.isDirectory)
            .map(entry => entry.name.toLowerCase());

          // Check if our target folder exists
          if (subFolders.includes(folderName)) {
            const componentPath = await join(componentsFolderPath, folderName);

            // Check if the component path exists
            if (await exists(componentPath)) {
              // Look for the golem YAML file in the component folder
              const files = await readDir(componentPath);
              const yamlFile = files
                .filter(entry => !entry.isDirectory)
                .map(entry => entry.name)
                .find(file => file === "golem.yaml" || file === "golem.yml");

              if (yamlFile) {
                return await join(componentPath, yamlFile);
              }
            }
          }
        } catch (error) {
          // Continue to the next components folder if this one fails
          console.warn(
            `Failed to read components folder ${componentsFolder}:`,
            error,
          );
        }
      }

      // Component folder isn't found in any components-* directory
      toast({
        title: "Error finding Component Manifest",
        description:
          "Could not find component golem.yaml for matched component in this app",
        variant: "destructive",
        duration: 5000,
      });
    } catch (error) {
      throw new Error(`Failed to scan app folder: ${error}`);
    }

    throw new Error(`Error finding Component Manifest`);
  }

  public async getComponentManifest(
    appId: string,
    componentId: string,
  ): Promise<GolemApplicationManifest> {
    const component = await this.getComponentById(appId, componentId);
    let componentYamlPath = await this.getComponentYamlPath(
      appId,
      component.componentName!,
    );
    let rawYaml = await readTextFile(componentYamlPath);

    return parse(rawYaml) as GolemApplicationManifest;
  }

  public async saveComponentManifest(
    appId: string,
    componentId: string,
    manifest: string,
  ): Promise<boolean> {
    const component = await this.getComponentById(appId, componentId);
    let componentYamlPath = await this.getComponentYamlPath(
      appId,
      component.componentName!,
    );
    // Write the YAML string to the file
    await writeTextFile(componentYamlPath, manifest);

    return true;
  }

  public async saveAppManifest(
    appId: string,
    manifest: string,
  ): Promise<boolean> {
    const app = await settingsService.getAppById(appId);
    if (!app) {
      throw new Error("App not found");
    }
    let appManifestPath = await join(app.folderLocation, "golem.yaml");
    await writeTextFile(appManifestPath, manifest);

    return true;
  }

  public getComponentByIdAndVersion = async (
    appId: string,
    componentId: string,
    version: number,
  ) => {
    const r = (await this.callCLI(appId, "component", ["get"])) as Component[];
    return r.find(
      c => c.componentId === componentId && c.componentVersion === version,
    );
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

  public updateComponent = async (componentId: string, form: FormData) => {
    console.log(componentId, form);
  };

  public deletePluginToComponent = async (
    id: string,
    installation_id: string,
  ) => {
    console.log(id, installation_id);
  };

  public addPluginToComponent = async (id: string, form: any) => {
    // return await this.callApi(
    //   ENDPOINT.addPluginToComponent(id),
    //   "POST",
    //   JSON.stringify(form),
    // );
    console.log(id, form);
  };

  public upgradeWorker = async (
    appId: string,
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
    let component = await this.getComponentById(appId, componentId);
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

  public getApiList = async (appId: string): Promise<HttpApiDefinition[]> => {
    let result: HttpApiDefinition[] = [];
    // we get it on a per-component basis
    let components = await this.getComponents(appId);
    for (const component of components) {
      try {
        let manifest = await this.getComponentManifest(
          appId,
          component.componentId!,
        );
        let APIList = manifest.httpApi;
        if (APIList) {
          for (const apiListKey in APIList.definitions) {
            let data = APIList.definitions[apiListKey];
            data.id = apiListKey;
            data.componentId = component.componentId;
            result.push(data);
          }
        }
      } catch (e) {
        console.error(e, component.componentName);
      }
    }
    // find in app's golem.yaml

    return result;
  };

  public getApi = async (
    appId: string,
    name: string,
  ): Promise<HttpApiDefinition[]> => {
    const ApiList = await this.getApiList(appId);
    const Api = ApiList.filter(a => a.id == name);
    if (!Api) {
      throw new Error("Api not found");
    }
    return Api;
  };

  public createApi = async (payload: HttpApiDefinition) => {
    // should use a YAML file
    // const r = await this.callApi(
    //   ENDPOINT.createApi(),
    //   "POST",
    //   JSON.stringify(payload),
    // );
    // return r;

    console.log(payload);
  };

  public deleteApi = async (appId: string, id: string, version: string) => {
    return await this.callCLI(appId, "api", [
      "definition",
      "delete",
      `--id=${id}`,
      `--version=${version}`,
    ]);
  };

  public putApi = async (
    id: string,
    version: string,
    payload: HttpApiDefinition,
  ) => {
    // should use YAML
    // const r = await this.callApi(
    //   ENDPOINT.putApi(id, version),
    //   "PUT",
    //   JSON.stringify(payload),
    // );
    // return r;

    console.log(id, payload, version);
  };

  public postApi = async (payload: Api) => {
    // const r = await this.callApi(
    //   ENDPOINT.postApi(),
    //   "POST",
    //   JSON.stringify(payload),
    // );
    // return r;

    console.log(payload);
  };

  public getParticularWorker = async (
    appId: string,
    componentId: string,
    workerName: string,
  ) => {
    const component = await this.getComponentById(appId, componentId);
    return await this.callCLI(appId, "worker", [
      "get",
      `${component?.componentName}/${workerName}`,
    ]);
  };

  public interruptWorker = async (
    appId: string,
    componentId: string,
    workerName: string,
  ) => {
    const component = await this.getComponentById(appId, componentId);
    const fullWorkerName = `${component?.componentName}/${workerName}`;
    return await this.callCLI(appId, "worker", ["interrupt", fullWorkerName]);
  };

  public resumeWorker = async (
    appId: string,
    componentId: string,
    workerName: string,
  ) => {
    const component = await this.getComponentById(appId, componentId);
    const fullWorkerName = `${component?.componentName}/${workerName}`;
    return await this.callCLI(appId, "worker", ["resume", fullWorkerName]);
  };

  public invokeWorkerAwait = async (
    appId: string,
    componentId: string,
    workerName: string,
    functionName: string,
    payload: any,
  ) => {
    // Get component name for proper worker identification
    const component = await this.getComponentById(appId, componentId);
    const fullWorkerName = `${component?.componentName}/${workerName}`;

    // Convert payload to individual WAVE-formatted arguments
    // Handle both old format (raw values) and new format (with type info)
    let waveArgs: string[];
    if (payload.params && payload.params.length > 0 && payload.params[0].typ) {
      waveArgs = payload.params.map((param: { value: any; typ: Parameter }) =>
        convertToWaveFormatWithType(param.value, {
          typ: param.typ,
          name: "",
          type: param.typ.type,
        }),
      );
    } else {
      // empty value
      waveArgs = [];
    }

    return await this.callCLI(appId, "worker", [
      "invoke",
      fullWorkerName,
      functionName,
      ...waveArgs,
    ]);
  };

  public invokeEphemeralAwait = async (
    appId: string,
    componentId: string,
    functionName: string,
    payload: any,
  ) => {
    // Get component name for ephemeral worker identification
    const component = await this.getComponentById(appId, componentId);
    const ephemeralWorkerName = `${component?.componentName}/-`;

    // Convert payload to individual WAVE-formatted arguments
    // Handle both old format (raw values) and new format (with type info)
    let waveArgs: string[];
    if (payload.params && payload.params.length > 0 && payload.params[0].typ) {
      // New format with type information
      // Filter out null option type parameters
      const filteredParams = payload.params.filter((param: any) => {
        if (
          param.typ?.type === "option" &&
          (param.value === null || param.value === undefined)
        ) {
          console.log("Skipping null option parameter:", param.name);
          return false;
        }
        return true;
      });

      waveArgs = filteredParams.map((param: any) =>
        convertToWaveFormatWithType(param.value, { typ: param.typ }),
      );
    } else {
      // Old format - raw values
      waveArgs = convertValuesToWaveArgs(payload);
    }

    return await this.callCLI(appId, "worker", [
      "invoke",
      ephemeralWorkerName,
      functionName,
      ...waveArgs,
    ]);
  };

  public getDeploymentApi = async (appId: string) => {
    return await this.callCLI(appId, "api", ["deployment", "list"]);
  };

  public deleteDeployment = async (appId: string, subdomain: string) => {
    return await this.callCLI(appId, "api", [
      "deployment",
      "delete",
      subdomain,
    ]);
  };

  public createDeployment = async (appId: string, subdomain?: string) => {
    const params = ["deployment", "deploy"];
    if (subdomain) {
      params.push(subdomain);
    }
    return await this.callCLI(appId, "api", params);
  };

  public getOplog = async (
    appId: string,
    componentId: string,
    workerName: string,
    searchQuery: string,
  ) => {
    // Get component name for proper worker identification
    const component = await this.getComponentById(appId, componentId);
    const fullWorkerName = `${component?.componentName}/${workerName}`;

    const r = await this.callCLI(appId, "worker", [
      "oplog",
      fullWorkerName,
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
          acc[key].versionList.push(componentVersion!);
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
    //  we use the "invoke" here to call a special command that calls golem CLI for us
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
      throw new Error("Error in calling golem CLI: " + String(e));
    }

    let parsedResult;
    const match = result.match(/(\[.*]|\{.*})/s);
    if (match) {
      try {
        parsedResult = JSON.parse(match[0]);
      } catch (e) {
        // some actions do not return JSON
      }
    }
    return parsedResult || true;
  };

  private callCLIWithLogs = async (
    appId: string,
    command: string,
    subcommands: string[],
  ): Promise<{ result: any; logs: string; success: boolean }> => {
    // find folder location
    const app = await settingsService.getAppById(appId);
    if (!app) {
      throw new Error("App not found");
    }
    //  we use the "invoke" here to call a special command that calls golem CLI for us
    let result: string;
    let success = true;

    try {
      result = await invoke("call_golem_command", {
        command,
        subcommands,
        folderPath: app.folderLocation,
      });
    } catch (e) {
      success = false;
      result = String(e);
    }

    let parsedResult;
    const match = result.match(/(\[.*]|\{.*})/s);
    if (match) {
      try {
        parsedResult = JSON.parse(match[0]);
      } catch (e) {
        // some actions do not return JSON
      }
    }

    return {
      result: parsedResult || true,
      logs: result,
      success,
    };
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

  public async createApiVersion(appId: string, payload: HttpApiDefinition) {
    // We need to know if the definition came from a component and store it there
    const app = await settingsService.getAppById(appId);
    let yamlToUpdate = app!.golemYamlLocation;

    if (payload.componentId) {
      const component = await this.getComponentById(appId, payload.componentId);
      yamlToUpdate = await this.getComponentYamlPath(
        appId,
        component.componentName!,
      );
    }

    // Now load the YAML into memory, update and save
    const rawYaml = await readTextFile(yamlToUpdate);

    // Parse as Document to preserve comments and formatting
    const manifest: Document = parseDocument(rawYaml);

    // Type-safe access to the parsed content
    // const manifestData = manifest.toJS() as GolemApplicationManifest;

    // Get or create httpApi section
    let httpApi = manifest.get("httpApi") as YAMLMap | undefined;
    if (!httpApi) {
      // Create new httpApi section if it doesn't exist
      manifest.set("httpApi", {});
      httpApi = manifest.get("httpApi") as YAMLMap;
    }

    // Get or create definitions section
    let definitions = httpApi.get("definitions") as YAMLMap | undefined;
    if (!definitions) {
      // Create new definitions section if it doesn't exist
      httpApi.set("definitions", {});
      definitions = httpApi.get("definitions") as YAMLMap;
    }

    // Add or update the API definition
    definitions.set(payload.id!, serializeHttpApiDefinition(payload));

    // Save config back
    if (payload.componentId) {
      await this.saveComponentManifest(
        appId,
        payload.componentId,
        manifest.toString(),
      );
    } else {
      await this.saveAppManifest(appId, manifest.toString());
    }
  }

  public buildApp = async (appId: string, componentNames?: string[]) => {
    const subcommands = ["build"];
    if (componentNames && componentNames.length > 0) {
      subcommands.push(...componentNames);
    }
    return await this.callCLIWithLogs(appId, "app", subcommands);
  };

  public updateWorkers = async (
    appId: string,
    componentNames?: string[],
    updateMode: string = "auto",
  ) => {
    const subcommands = ["update-workers"];
    if (updateMode) {
      subcommands.push("--update-mode", updateMode);
    }
    if (componentNames && componentNames.length > 0) {
      subcommands.push(...componentNames);
    }
    return await this.callCLIWithLogs(appId, "app", subcommands);
  };

  public deployWorkers = async (
    appId: string,
    componentNames?: string[],
    updateWorkers?: boolean,
  ) => {
    const subcommands = ["deploy"];
    if (updateWorkers) {
      subcommands.push("--update-workers");
    }
    if (componentNames && componentNames.length > 0) {
      subcommands.push(...componentNames);
    }
    return await this.callCLIWithLogs(appId, "app", subcommands);
  };

  public cleanApp = async (appId: string, componentNames?: string[]) => {
    const subcommands = ["clean"];
    if (componentNames && componentNames.length > 0) {
      subcommands.push(...componentNames);
    }
    return await this.callCLIWithLogs(appId, "app", subcommands);
  };

  public getAppYamlContent = async (appId: string): Promise<string> => {
    const app = await settingsService.getAppById(appId);
    if (!app) {
      throw new Error("App not found");
    }
    const appManifestPath = await join(app.folderLocation, "golem.yaml");
    if (await exists(appManifestPath)) {
      return await readTextFile(appManifestPath);
    }
    const appManifestPathYml = await join(app.folderLocation, "golem.yml");
    if (await exists(appManifestPathYml)) {
      return await readTextFile(appManifestPathYml);
    }
    throw new Error("App manifest file not found");
  };

  public getComponentYamlContent = async (
    appId: string,
    componentName: string,
  ): Promise<string> => {
    const componentYamlPath = await this.getComponentYamlPath(
      appId,
      componentName,
    );
    return await readTextFile(componentYamlPath);
  };
}
