import { CLIService } from "./client/cli-service";
import { ComponentService } from "./client/component-service";
import { WorkerService } from "./client/worker-service";
import { APIService } from "./client/api-service";
import { PluginService } from "./client/plugin-service";
import { DeploymentService } from "./client/deployment-service";
import { AppService } from "./client/app-service";
import { ManifestService } from "./client/manifest-service";

export class Service {
  public baseUrl: string;
  private cliService: CLIService;
  private componentService: ComponentService;
  private workerService: WorkerService;
  private apiService: APIService;
  private pluginService: PluginService;
  private deploymentService: DeploymentService;
  private appService: AppService;
  private manifestService: ManifestService;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
    
    // Initialize services in the correct order to handle dependencies
    this.cliService = new CLIService();
    this.componentService = new ComponentService(this.cliService);
    this.manifestService = new ManifestService(this.cliService);
    this.workerService = new WorkerService(this.cliService, this.componentService);
    this.apiService = new APIService(this.cliService, this.componentService, this.manifestService);
    this.pluginService = new PluginService(this.cliService);
    this.deploymentService = new DeploymentService(this.cliService);
    this.appService = new AppService(this.cliService);
  }

  // Health check methods
  public checkHealth = async (): Promise<Boolean> => {
    return this.appService.checkHealth();
  };

  // Component methods
  public getComponents = async (appId: string) => {
    return this.componentService.getComponents(appId);
  };

  public getComponentById = async (appId: string, componentId: string) => {
    return this.componentService.getComponentById(appId, componentId);
  };

  public getComponentByIdAndVersion = async (appId: string, componentId: string, version: number) => {
    return this.componentService.getComponentByIdAndVersion(appId, componentId, version);
  };

  public createComponent = async (appId: string, name: string, template: string) => {
    return this.componentService.createComponent(appId, name, template);
  };

  public getComponentByName = async (appId: string, name: string) => {
    return this.componentService.getComponentByName(appId, name);
  };

  public updateComponent = async (componentId: string, form: FormData) => {
    return this.componentService.updateComponent(componentId, form);
  };

  public deletePluginToComponent = async (id: string, installation_id: string) => {
    return this.componentService.deletePluginToComponent(id, installation_id);
  };

  public addPluginToComponent = async (id: string, form: any) => {
    return this.componentService.addPluginToComponent(id, form);
  };

  public getComponentByIdAsKey = async (appId: string) => {
    return this.componentService.getComponentByIdAsKey(appId);
  };

  // Worker methods
  public upgradeWorker = async (appId: string, componentName: string, workerName: string, version: number, upgradeType: string) => {
    return this.workerService.upgradeWorker(appId, componentName, workerName, version, upgradeType);
  };

  public findWorker = async (appId: string, componentId: string, param = { count: 100, precise: true }) => {
    return this.workerService.findWorker(appId, componentId, param);
  };

  public deleteWorker = async (appId: string, componentId: string, workerName: string) => {
    return this.workerService.deleteWorker(appId, componentId, workerName);
  };

  public createWorker = async (appId: string, componentID: string, name: string) => {
    return this.workerService.createWorker(appId, componentID, name);
  };

  public getParticularWorker = async (appId: string, componentId: string, workerName: string) => {
    return this.workerService.getParticularWorker(appId, componentId, workerName);
  };

  public interruptWorker = async (appId: string, componentId: string, workerName: string) => {
    return this.workerService.interruptWorker(appId, componentId, workerName);
  };

  public resumeWorker = async (appId: string, componentId: string, workerName: string) => {
    return this.workerService.resumeWorker(appId, componentId, workerName);
  };

  public invokeWorkerAwait = async (appId: string, componentId: string, workerName: string, functionName: string, payload: any) => {
    return this.workerService.invokeWorkerAwait(appId, componentId, workerName, functionName, payload);
  };

  public invokeEphemeralAwait = async (appId: string, componentId: string, functionName: string, payload: any) => {
    return this.workerService.invokeEphemeralAwait(appId, componentId, functionName, payload);
  };

  public getOplog = async (appId: string, componentId: string, workerName: string, searchQuery: string) => {
    return this.workerService.getOplog(appId, componentId, workerName, searchQuery);
  };

  // API methods
  public getApiList = async (appId: string) => {
    return this.apiService.getApiList(appId);
  };

  public getApi = async (appId: string, name: string) => {
    return this.apiService.getApi(appId, name);
  };

  public createApi = async (appId: string, payload: any) => {
    return this.apiService.createApi(appId, payload);
  };

  public deleteApi = async (appId: string, id: string, version: string) => {
    return this.apiService.deleteApi(appId, id, version);
  };

  public putApi = async (id: string, version: string, payload: any) => {
    return this.apiService.putApi(id, version, payload);
  };

  public postApi = async (payload: any) => {
    return this.apiService.postApi(payload);
  };

  public createApiVersion = async (appId: string, payload: any) => {
    return this.apiService.createApiVersion(appId, payload);
  };

  // Plugin methods
  public getPlugins = async (appId: string) => {
    return this.pluginService.getPlugins(appId);
  };

  public getPluginByName = async (appId: string, name: string) => {
    return this.pluginService.getPluginByName(appId, name);
  };

  public createPlugin = async (appId: string, pluginData: any) => {
    return this.pluginService.createPlugin(appId, pluginData);
  };

  public registerPlugin = async (appId: string, manifestFileLocation: string) => {
    return this.pluginService.registerPlugin(appId, manifestFileLocation);
  };

  public deletePlugin = async (appId: string, name: string, version: string) => {
    return this.pluginService.deletePlugin(appId, name, version);
  };

  // Deployment methods
  public getDeploymentApi = async (appId: string) => {
    return this.deploymentService.getDeploymentApi(appId);
  };

  public deleteDeployment = async (appId: string, subdomain: string) => {
    return this.deploymentService.deleteDeployment(appId, subdomain);
  };

  public createDeployment = async (appId: string, subdomain?: string) => {
    return this.deploymentService.createDeployment(appId, subdomain);
  };

  // App methods
  public buildApp = async (appId: string, componentNames?: string[]) => {
    return this.appService.buildApp(appId, componentNames);
  };

  public updateWorkers = async (appId: string, componentNames?: string[], updateMode: string = "auto") => {
    return this.appService.updateWorkers(appId, componentNames, updateMode);
  };

  public deployWorkers = async (appId: string, componentNames?: string[], updateWorkers?: boolean) => {
    return this.appService.deployWorkers(appId, componentNames, updateWorkers);
  };

  public cleanApp = async (appId: string, componentNames?: string[]) => {
    return this.appService.cleanApp(appId, componentNames);
  };

  // Manifest methods
  public getComponentYamlPath = async (appId: string, componentName: string) => {
    return this.manifestService.getComponentYamlPath(appId, componentName);
  };

  public getAppYamlPath = async (appId: string) => {
    return this.manifestService.getAppYamlPath(appId);
  };

  public getComponentManifest = async (appId: string, componentId: string) => {
    return this.manifestService.getComponentManifest(appId, componentId);
  };

  public getAppManifest = async (appId: string) => {
    return this.manifestService.getAppManifest(appId);
  };

  public saveComponentManifest = async (appId: string, componentId: string, manifest: string) => {
    return this.manifestService.saveComponentManifest(appId, componentId, manifest);
  };

  public saveAppManifest = async (appId: string, manifest: string) => {
    return this.manifestService.saveAppManifest(appId, manifest);
  };

  public getAppYamlContent = async (appId: string) => {
    return this.manifestService.getAppYamlContent(appId);
  };

  public getComponentYamlContent = async (appId: string, componentName: string) => {
    return this.manifestService.getComponentYamlContent(appId, componentName);
  };
}
