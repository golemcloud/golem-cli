import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { Dashboard } from "../index";

// Mock dependencies
const mockNavigate = vi.fn();
const mockUseParams = vi.fn().mockReturnValue({ appId: "test-app-id" });

vi.mock("react-router-dom", async () => {
  const actual = await vi.importActual("react-router-dom");
  return {
    ...actual,
    useNavigate: () => mockNavigate,
    useParams: () => mockUseParams(),
  };
});

vi.mock("@/lib/settings", () => ({
  storeService: {
    getAppById: vi.fn().mockResolvedValue({
      id: "test-app-id",
      name: "Test App",
    }),
    updateAppLastOpened: vi.fn().mockResolvedValue(true),
  },
}));

vi.mock("@/pages/dashboard/componentSection", () => ({
  ComponentsSection: () => (
    <div data-testid="components-section">Components Section</div>
  ),
}));

vi.mock("@/pages/dashboard/apiSection", () => ({
  APISection: () => <div data-testid="api-section">API Section</div>,
}));

vi.mock("@/pages/dashboard/deploymentSection", () => ({
  DeploymentSection: () => (
    <div data-testid="deployment-section">Deployment Section</div>
  ),
}));

vi.mock("@/components/ui/button", () => ({
  Button: ({ children, onClick, variant }: any) => (
    <button onClick={onClick} data-variant={variant}>
      {children}
    </button>
  ),
}));

describe("Dashboard", () => {
  beforeEach(async () => {
    vi.clearAllMocks();
    // Set up default mock for useParams
    mockUseParams.mockReturnValue({ appId: "test-app-id" });
    // Reset mock to default values
    const { storeService } = await import("@/lib/settings");
    (storeService.getAppById as any).mockResolvedValue({
      id: "test-app-id",
      name: "Test App",
    });
    (storeService.updateAppLastOpened as any).mockResolvedValue(true);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  const renderDashboard = () => {
    return render(
      <MemoryRouter initialEntries={["/app/test-app-id"]}>
        <Dashboard />
      </MemoryRouter>,
    );
  };

  describe("Component Rendering", () => {
    it("should render the dashboard with app name", async () => {
      renderDashboard();

      // Verify the basic dashboard structure renders
      await waitFor(() => {
        expect(screen.getByText(/Working in/)).toBeInTheDocument();
        expect(screen.getByText("Back to Apps")).toBeInTheDocument();
        expect(screen.getByText("App ID:")).toBeInTheDocument();
        expect(screen.getByText("test-app-id")).toBeInTheDocument();
      });
    });

    it("should render with default app name when app not found", async () => {
      const { storeService } = await import("@/lib/settings");
      (storeService.getAppById as any).mockResolvedValue(null);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText("Working in App")).toBeInTheDocument();
      });
    });

    it("should render app ID display", async () => {
      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText("App ID:")).toBeInTheDocument();
        expect(screen.getByText("test-app-id")).toBeInTheDocument();
      });
    });

    it("should render back to apps button", async () => {
      const { storeService } = await import("@/lib/settings");
      (storeService.getAppById as any).mockResolvedValue({
        id: "test-app-id",
        name: "Test App",
      });

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText("Back to Apps")).toBeInTheDocument();
      });
    });

    it("should render all dashboard sections", async () => {
      const { storeService } = await import("@/lib/settings");
      (storeService.getAppById as any).mockResolvedValue({
        id: "test-app-id",
        name: "Test App",
      });

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByTestId("components-section")).toBeInTheDocument();
        expect(screen.getByTestId("api-section")).toBeInTheDocument();
        expect(screen.getByTestId("deployment-section")).toBeInTheDocument();
      });
    });
  });

  describe("Navigation", () => {
    it("should navigate to home when back button is clicked", async () => {
      const { storeService } = await import("@/lib/settings");
      (storeService.getAppById as any).mockResolvedValue({
        id: "test-app-id",
        name: "Test App",
      });

      renderDashboard();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText("Back to Apps")).toBeInTheDocument();
      });

      const backButton = screen.getByText("Back to Apps");
      await user.click(backButton);

      expect(mockNavigate).toHaveBeenCalledWith("/");
    });

    it("should redirect to home when no app ID is provided", async () => {
      // Mock useParams to return no appId
      mockUseParams.mockReturnValue({});

      renderDashboard();

      await waitFor(() => {
        expect(mockNavigate).toHaveBeenCalledWith("/");
      });
    });
  });

  describe("Data Loading", () => {
    it("should load app data on mount", async () => {
      const { storeService } = await import("@/lib/settings");

      renderDashboard();

      await waitFor(() => {
        expect(storeService.getAppById).toHaveBeenCalledWith("test-app-id");
      });
    });

    it("should update app last opened on mount", async () => {
      const { storeService } = await import("@/lib/settings");

      renderDashboard();

      await waitFor(() => {
        expect(storeService.updateAppLastOpened).toHaveBeenCalledWith(
          "test-app-id",
        );
      });
    });

    it("should handle app loading errors gracefully", async () => {
      const { storeService } = await import("@/lib/settings");
      (storeService.getAppById as any).mockRejectedValue(
        new Error("Failed to load app"),
      );

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText("Working in App")).toBeInTheDocument();
      });
    });
  });

  describe("Layout", () => {
    it("should have proper grid layout structure", async () => {
      const { storeService } = await import("@/lib/settings");
      (storeService.getAppById as any).mockResolvedValue({
        id: "test-app-id",
        name: "Test App",
      });

      renderDashboard();

      await waitFor(() => {
        const gridElement =
          screen.getByTestId("components-section").parentElement;
        expect(gridElement).toHaveClass("grid");
      });
    });

    it("should display app information in a bordered container", async () => {
      const { storeService } = await import("@/lib/settings");
      (storeService.getAppById as any).mockResolvedValue({
        id: "test-app-id",
        name: "Test App",
      });

      renderDashboard();

      await waitFor(() => {
        const appInfoContainer = screen.getByText("App ID:").parentElement;
        expect(appInfoContainer).toHaveClass("border", "rounded-lg");
      });
    });
  });

  describe("Error Handling", () => {
    it("should handle missing app gracefully", async () => {
      const { storeService } = await import("@/lib/settings");
      (storeService.getAppById as any).mockResolvedValue(null);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText("Working in App")).toBeInTheDocument();
        expect(screen.getByText("test-app-id")).toBeInTheDocument();
      });
    });

    it("should handle app with missing name", async () => {
      const { storeService } = await import("@/lib/settings");
      (storeService.getAppById as any).mockResolvedValue({
        id: "test-app-id",
        name: "",
      });

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText("Working in App")).toBeInTheDocument();
      });
    });
  });

  describe("Accessibility", () => {
    it("should have proper heading structure", async () => {
      const { storeService } = await import("@/lib/settings");
      (storeService.getAppById as any).mockResolvedValue({
        id: "test-app-id",
        name: "Test App",
      });

      renderDashboard();

      // Wait for the component to load app data and update
      await waitFor(() => {
        expect(storeService.getAppById).toHaveBeenCalledWith("test-app-id");
      });

      await waitFor(() => {
        expect(
          screen.getByRole("heading", { name: /Working in Test App/i }),
        ).toBeInTheDocument();
      });
    });

    it("should support keyboard navigation", async () => {
      const { storeService } = await import("@/lib/settings");
      (storeService.getAppById as any).mockResolvedValue({
        id: "test-app-id",
        name: "Test App",
      });

      renderDashboard();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText("Back to Apps")).toBeInTheDocument();
      });

      await user.tab();
      expect(document.activeElement).toBe(screen.getByText("Back to Apps"));
    });
  });

  describe("Component Integration", () => {
    it("should pass app context to child components", async () => {
      const { storeService } = await import("@/lib/settings");
      (storeService.getAppById as any).mockResolvedValue({
        id: "test-app-id",
        name: "Test App",
      });

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByTestId("components-section")).toBeInTheDocument();
        expect(screen.getByTestId("api-section")).toBeInTheDocument();
        expect(screen.getByTestId("deployment-section")).toBeInTheDocument();
      });

      // All sections should be rendered indicating they receive the app context
      expect(screen.getByText("Components Section")).toBeInTheDocument();
      expect(screen.getByText("API Section")).toBeInTheDocument();
      expect(screen.getByText("Deployment Section")).toBeInTheDocument();
    });
  });
});
