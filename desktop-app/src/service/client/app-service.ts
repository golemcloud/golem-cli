import { CLIService } from "./cli-service";

export class AppService {
  private cliService: CLIService;

  constructor(cliService: CLIService) {
    this.cliService = cliService;
  }

  /**
   * checkHealth: Check if the service is healthy
   * @returns {Promise<Boolean>}
   */
  public checkHealth = async (): Promise<Boolean> => {
    return Promise.resolve(true);
  };

  public buildApp = async (appId: string, componentNames?: string[]) => {
    const subcommands = ["build"];
    if (componentNames && componentNames.length > 0) {
      subcommands.push(...componentNames);
    }
    return await this.cliService.callCLIWithLogs(appId, "app", subcommands);
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
    return await this.cliService.callCLIWithLogs(appId, "app", subcommands);
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
    return await this.cliService.callCLIWithLogs(appId, "app", subcommands);
  };

  public cleanApp = async (appId: string, componentNames?: string[]) => {
    const subcommands = ["clean"];
    if (componentNames && componentNames.length > 0) {
      subcommands.push(...componentNames);
    }
    return await this.cliService.callCLIWithLogs(appId, "app", subcommands);
  };
}
