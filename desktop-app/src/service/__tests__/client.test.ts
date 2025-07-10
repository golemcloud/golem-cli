import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { Service } from "../client";
import { toast } from "@/hooks/use-toast";
import { ComponentType } from "@/types/component";

// Mock dependencies
vi.mock("@/lib/settings", () => ({
  settingsService: {
    getAppById: vi.fn(),
  },
}));

vi.mock("@/hooks/use-toast", () => ({
  toast: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-fs", () => ({
  exists: vi.fn(),
  readDir: vi.fn(),
  readTextFile: vi.fn(),
  writeTextFile: vi.fn(),
}));

vi.mock("@tauri-apps/api/path", () => ({
  join: vi.fn(),
}));

vi.mock("yaml", () => ({
  parse: vi.fn(),
  parseDocument: vi.fn(),
  stringify: vi.fn(),
}));

describe("Service", () => {
  let service: Service;
  const mockApp = {
    id: "test-app-id",
    folderLocation: "/test/folder",
    golemYamlLocation: "/test/folder/golem.yaml",
  };

  beforeEach(() => {
    service = new Service("http://localhost:3000");
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("constructor", () => {
    it("should initialize with baseUrl", () => {
      expect(service.baseUrl).toBe("http://localhost:3000");
    });
  });

  describe("checkHealth", () => {
    it("should return true", async () => {
      const result = await service.checkHealth();
      expect(result).toBe(true);
    });
  });

  describe("getComponents", () => {
    it("should call CLI and return components", async () => {
      const mockComponents = [
        { componentId: "comp1", componentName: "Component 1" },
        { componentId: "comp2", componentName: "Component 2" },
      ];

      const callCLISpy = vi
        .spyOn(service as any, "callCLI")
        .mockResolvedValue(mockComponents);

      const result = await service.getComponents("test-app-id");

      expect(callCLISpy).toHaveBeenCalledWith("test-app-id", "component", [
        "list",
      ]);
      expect(result).toEqual(mockComponents);
    });
  });

  describe("getComponentById", () => {
    it("should return component when found", async () => {
      const mockComponents = [
        { componentId: "comp1", componentName: "Component 1" },
        { componentId: "comp2", componentName: "Component 2" },
      ];

      const callCLISpy = vi
        .spyOn(service as any, "callCLI")
        .mockResolvedValue(mockComponents);

      const result = await service.getComponentById("test-app-id", "comp1");

      expect(callCLISpy).toHaveBeenCalledWith("test-app-id", "component", [
        "list",
      ]);
      expect(result).toEqual(mockComponents[0]);
    });

    it("should throw error when component not found", async () => {
      const mockComponents = [
        { componentId: "comp1", componentName: "Component 1" },
      ];

      vi.spyOn(service as any, "callCLI").mockResolvedValue(mockComponents);

      await expect(
        service.getComponentById("test-app-id", "nonexistent"),
      ).rejects.toThrow("Could not find component");
    });
  });

  describe("getComponentYamlPath", () => {
    it("should find component yaml path successfully", async () => {
      const { settingsService } = await import("@/lib/settings");
      const { readDir, exists } = await import("@tauri-apps/plugin-fs");
      const { join } = await import("@tauri-apps/api/path");

      (settingsService.getAppById as any).mockResolvedValue(mockApp);
      (readDir as any)
        .mockResolvedValueOnce([
          { name: "components-main", isDirectory: true },
          { name: "other-folder", isDirectory: true },
        ])
        .mockResolvedValueOnce([{ name: "test-component", isDirectory: true }])
        .mockResolvedValueOnce([
          { name: "golem.yaml", isDirectory: false },
          { name: "src", isDirectory: true },
        ]);
      (exists as any).mockResolvedValue(true);
      (join as any)
        .mockResolvedValueOnce("/test/folder/components-main")
        .mockResolvedValueOnce("/test/folder/components-main/test-component")
        .mockResolvedValueOnce(
          "/test/folder/components-main/test-component/golem.yaml",
        );

      const result = await service.getComponentYamlPath(
        "test-app-id",
        "test:component",
      );

      expect(result).toBe(
        "/test/folder/components-main/test-component/golem.yaml",
      );
    });

    it("should throw error when app not found", async () => {
      const { settingsService } = await import("@/lib/settings");
      (settingsService.getAppById as any).mockResolvedValue(null);

      await expect(
        service.getComponentYamlPath("test-app-id", "test-component"),
      ).rejects.toThrow("App not found");
    });

    it("should show toast and throw error when component not found", async () => {
      const { settingsService } = await import("@/lib/settings");
      const { readDir } = await import("@tauri-apps/plugin-fs");

      (settingsService.getAppById as any).mockResolvedValue(mockApp);
      (readDir as any).mockResolvedValueOnce([]);

      await expect(
        service.getComponentYamlPath("test-app-id", "nonexistent"),
      ).rejects.toThrow("Error finding Component Manifest");

      expect(toast).toHaveBeenCalledWith({
        title: "Error finding Component Manifest",
        description:
          "Could not find component golem.yaml for matched component in this app",
        variant: "destructive",
        duration: 5000,
      });
    });
  });

  describe("createComponent", () => {
    it("should create component successfully", async () => {
      const callCLISpy = vi
        .spyOn(service as any, "callCLI")
        .mockResolvedValue(true);

      await service.createComponent("test-app-id", "test-component", "rust");

      expect(callCLISpy).toHaveBeenCalledWith("test-app-id", "component", [
        "new",
        "rust",
        "test-component",
      ]);
    });
  });

  describe("createWorker", () => {
    it("should create worker successfully", async () => {
      const mockComponent = {
        componentId: "comp1",
        componentName: "test-component",
      };
      const getComponentSpy = vi
        .spyOn(service, "getComponentById")
        .mockResolvedValue(mockComponent);
      const callCLISpy = vi
        .spyOn(service as any, "callCLI")
        .mockResolvedValue(true);

      await service.createWorker("test-app-id", "comp1", "worker-name");

      expect(getComponentSpy).toHaveBeenCalledWith("test-app-id", "comp1");
      expect(callCLISpy).toHaveBeenCalledWith("test-app-id", "worker", [
        "new",
        "test-component/worker-name",
      ]);
    });
  });

  describe("deleteWorker", () => {
    it("should delete worker successfully", async () => {
      const mockComponent = {
        componentId: "comp1",
        componentName: "test-component",
      };
      const getComponentSpy = vi
        .spyOn(service, "getComponentById")
        .mockResolvedValue(mockComponent);
      const callCLISpy = vi
        .spyOn(service as any, "callCLI")
        .mockResolvedValue(true);

      await service.deleteWorker("test-app-id", "comp1", "worker-name");

      expect(getComponentSpy).toHaveBeenCalledWith("test-app-id", "comp1");
      expect(callCLISpy).toHaveBeenCalledWith("test-app-id", "worker", [
        "delete",
        "test-component/worker-name",
      ]);
    });
  });

  describe("getApiList", () => {
    it("should return API list from multiple components", async () => {
      const mockComponents = [
        { componentId: "comp1", componentName: "Component 1" },
        { componentId: "comp2", componentName: "Component 2" },
      ];

      const mockManifest1 = {
        httpApi: {
          definitions: {
            api1: { version: "1.0.0" },
            api2: { version: "1.0.0" },
          },
        },
      };

      const mockManifest2 = {
        httpApi: {
          definitions: {
            api3: { version: "1.0.0" },
          },
        },
      };

      vi.spyOn(service, "getComponents").mockResolvedValue(mockComponents);
      vi.spyOn(service, "getComponentManifest")
        .mockResolvedValueOnce(mockManifest1)
        .mockResolvedValueOnce(mockManifest2);

      const result = await service.getApiList("test-app-id");

      expect(result).toHaveLength(3);
      expect(result[0]).toEqual({
        id: "api1",
        version: "1.0.0",
        componentId: "comp1",
      });
      expect(result[1]).toEqual({
        id: "api2",
        version: "1.0.0",
        componentId: "comp1",
      });
      expect(result[2]).toEqual({
        id: "api3",
        version: "1.0.0",
        componentId: "comp2",
      });
    });

    it("should handle components without httpApi", async () => {
      const mockComponents = [
        { componentId: "comp1", componentName: "Component 1" },
      ];

      const mockManifest = {};

      vi.spyOn(service, "getComponents").mockResolvedValue(mockComponents);
      vi.spyOn(service, "getComponentManifest").mockResolvedValue(mockManifest);

      const result = await service.getApiList("test-app-id");

      expect(result).toHaveLength(0);
    });
  });

  describe("getComponentByIdAsKey", () => {
    it("should organize components by ID with version lists", async () => {
      const mockComponents = [
        {
          componentId: "comp1",
          componentName: "Component 1",
          componentType: "Durable" as ComponentType,
          componentVersion: 1,
        },
        {
          componentId: "comp1",
          componentName: "Component 1",
          componentType: "Durable" as ComponentType,
          componentVersion: 2,
        },
        {
          componentId: "comp2",
          componentName: "Component 2",
          componentType: "Ephemeral" as ComponentType,
          componentVersion: 1,
        },
      ];

      vi.spyOn(service, "getComponents").mockResolvedValue(mockComponents);

      const result = await service.getComponentByIdAsKey("test-app-id");

      expect(result).toEqual({
        comp1: {
          componentName: "Component 1",
          componentId: "comp1",
          componentType: "Durable" as ComponentType,
          versions: [mockComponents[0], mockComponents[1]],
          versionList: [1, 2],
        },
        comp2: {
          componentName: "Component 2",
          componentId: "comp2",
          componentType: "Ephemeral" as ComponentType,
          versions: [mockComponents[2]],
          versionList: [1],
        },
      });
    });
  });

  describe("private callCLI", () => {
    it("should call CLI successfully and parse faulty JSON response", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { settingsService } = await import("@/lib/settings");

      (settingsService.getAppById as any).mockResolvedValue(mockApp);
      (invoke as any).mockResolvedValue('{"result": "success"} 899');

      const result = await (service as any).callCLI(
        "test-app-id",
        "component",
        ["list"],
      );

      expect(invoke).toHaveBeenCalledWith("call_golem_command", {
        command: "component",
        subcommands: ["list"],
        folderPath: "/test/folder",
      });
      expect(result).toEqual({ result: "success" });
    });

    it("should call CLI successfully and parse correct JSON response", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { settingsService } = await import("@/lib/settings");

      (settingsService.getAppById as any).mockResolvedValue(mockApp);
      (invoke as any).mockResolvedValue(
        '{"hash":"cee96ea1","configHash":"69ca925a","lockfileHash":"69742859","browserHash":"7a651483","optimized":{},"chunks":{}}',
      );

      const result = await (service as any).callCLI(
        "test-app-id",
        "component",
        ["list"],
      );

      expect(invoke).toHaveBeenCalledWith("call_golem_command", {
        command: "component",
        subcommands: ["list"],
        folderPath: "/test/folder",
      });
      expect(result).toEqual({
        hash: "cee96ea1",
        configHash: "69ca925a",
        lockfileHash: "69742859",
        browserHash: "7a651483",
        optimized: {},
        chunks: {},
      });
    });

    it("should return true for non-JSON responses", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { settingsService } = await import("@/lib/settings");

      (settingsService.getAppById as any).mockResolvedValue(mockApp);
      (invoke as any).mockResolvedValue("Command executed successfully");

      const result = await (service as any).callCLI(
        "test-app-id",
        "component",
        ["list"],
      );

      expect(result).toBe(true);
    });

    it("should handle CLI errors and show toast", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { settingsService } = await import("@/lib/settings");

      (settingsService.getAppById as any).mockResolvedValue(mockApp);
      (invoke as any).mockRejectedValue(new Error("CLI failed"));

      await expect(
        (service as any).callCLI("test-app-id", "component", ["list"]),
      ).rejects.toThrow("Error in calling golem CLI");

      expect(toast).toHaveBeenCalledWith({
        title: "Error in calling golem CLI",
        description: "Error: CLI failed",
        variant: "destructive",
        duration: 5000,
      });
    });

    it("should throw error when app not found", async () => {
      const { settingsService } = await import("@/lib/settings");
      (settingsService.getAppById as any).mockResolvedValue(null);

      await expect(
        (service as any).callCLI("test-app-id", "component", ["list"]),
      ).rejects.toThrow("App not found");
    });
  });
});
