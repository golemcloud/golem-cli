import { CLIService } from "./cli-service";

export class DeploymentService {
  private cliService: CLIService;

  constructor(cliService: CLIService) {
    this.cliService = cliService;
  }

  public getDeploymentApi = async (appId: string) => {
    return await this.cliService.callCLI(appId, "api", ["deployment", "list"]);
  };

  public deleteDeployment = async (appId: string, subdomain: string) => {
    return await this.cliService.callCLI(appId, "api", [
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
    return await this.cliService.callCLI(appId, "api", params);
  };
}
