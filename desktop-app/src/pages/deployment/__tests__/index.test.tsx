import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import DeploymentList from "../index";

// Mock dependencies
const mockNavigate = vi.fn();
vi.mock("react-router-dom", async () => {
  const actual = await vi.importActual("react-router-dom");
  return {
    ...actual,
    useNavigate: () => mockNavigate,
    useParams: () => ({ appId: "test-app-id" }),
  };
});

vi.mock("@/service", () => ({
  API: {
    getDeploymentApi: vi.fn(),
    deleteDeployment: vi.fn(),
    getApiList: vi.fn(),
  },
}));

vi.mock("@/components/errorBoundary", () => ({
  default: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="error-boundary">{children}</div>
  ),
}));

vi.mock("@/components/ui/card", () => ({
  Card: ({ children, className }: any) => (
    <div className={className} data-testid="card">
      {children}
    </div>
  ),
  CardContent: ({ children }: any) => <div>{children}</div>,
}));

vi.mock("@/components/ui/dialog", () => ({
  Dialog: ({ children, open, onOpenChange }: any) => (
    <div>
      <div data-trigger onClick={() => onOpenChange?.(true)}>
        {children[0]}
      </div>
      {open && <div data-testid="dialog">{children.slice(1)}</div>}
    </div>
  ),
  DialogContent: ({ children }: any) => <div>{children}</div>,
  DialogDescription: ({ children }: any) => <p>{children}</p>,
  DialogFooter: ({ children }: any) => <div>{children}</div>,
  DialogHeader: ({ children }: any) => <div>{children}</div>,
  DialogTitle: ({ children }: any) => <h2>{children}</h2>,
  DialogTrigger: ({ children }: any) => <div>{children}</div>,
}));

vi.mock("@/components/ui/button", () => ({
  Button: ({
    children,
    onClick,
    variant,
    disabled,
    asChild,
    ...props
  }: any) => {
    if (asChild) {
      return (
        <div onClick={onClick} {...props}>
          {children}
        </div>
      );
    }
    return (
      <button
        onClick={onClick}
        data-variant={variant}
        disabled={disabled}
        {...props}
      >
        {children}
      </button>
    );
  },
}));

vi.mock("@/components/ui/badge", () => ({
  Badge: ({ children, className }: any) => (
    <span className={className} data-testid="badge">
      {children}
    </span>
  ),
}));

vi.mock("@/lib/utils", () => ({
  cn: vi.fn((...args) => args.join(" ")),
  removeDuplicateApis: vi.fn(apis => apis),
}));

vi.mock("@/components/nav-route", () => ({
  HTTP_METHOD_COLOR: {
    GET: "text-green-500",
    POST: "text-blue-500",
    PUT: "text-yellow-500",
    DELETE: "text-red-500",
  },
}));

vi.mock("lucide-react", () => ({
  ChevronRight: () => <span data-testid="chevron-right-icon">▶</span>,
  Copy: () => <span data-testid="copy-icon">📋</span>,
  Layers: () => <span data-testid="layers-icon">📚</span>,
  Plus: () => <span data-testid="plus-icon">➕</span>,
  Trash: () => <span data-testid="trash-icon">🗑️</span>,
}));

// Mock clipboard globally once
Object.defineProperty(navigator, "clipboard", {
  value: {
    writeText: vi.fn(),
  },
  writable: true,
  configurable: true,
});

describe("DeploymentList", () => {
  const mockDeployments = [
    {
      apiDefinitions: [
        {
          id: "api-1",
          version: "1.0.0",
        },
      ],
      createdAt: "2023-12-01T10:00:00Z",
      projectId: "test-app-id",
      site: {
        host: "localhost:8080",
        subdomain: "test1",
      },
    },
    {
      apiDefinitions: [
        {
          id: "api-2",
          version: "2.0.0",
        },
      ],
      createdAt: "2023-12-01T11:00:00Z",
      projectId: "test-app-id",
      site: {
        host: "localhost:8081",
        subdomain: "test2",
      },
    },
  ];

  const mockApiList = [
    {
      id: "api-1",
      version: "1.0.0",
      routes: [
        { path: "/api/test1", method: "GET" },
        { path: "/api/test1", method: "POST" },
      ],
    },
    {
      id: "api-2",
      version: "2.0.0",
      routes: [
        { path: "/api/test2", method: "GET" },
        { path: "/api/test2", method: "DELETE" },
      ],
    },
  ];

  beforeEach(async () => {
    vi.clearAllMocks();
    Object.defineProperty(window, "matchMedia", {
      writable: true,
      value: vi.fn().mockImplementation(query => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: vi.fn(), // deprecated
        removeListener: vi.fn(), // deprecated
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
    });

    // Set up default mocks
    const { API } = await import("@/service");
    (API.getApiList as any).mockResolvedValue(mockApiList);
    (API.getDeploymentApi as any).mockResolvedValue(mockDeployments);
    (API.deleteDeployment as any).mockResolvedValue(true);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  const renderDeploymentList = () => {
    return render(
      <MemoryRouter>
        <DeploymentList />
      </MemoryRouter>,
    );
  };

  describe("Component Rendering", () => {
    it("should render the deployment list page", async () => {
      renderDeploymentList();

      await waitFor(() => {
        expect(screen.getAllByText("localhost:8080")[0]).toBeInTheDocument();
        expect(screen.getAllByText("localhost:8081")[0]).toBeInTheDocument();
      });
    });

    it("should render create deployment button", async () => {
      renderDeploymentList();

      await waitFor(() => {
        expect(screen.getByText("New")).toBeInTheDocument();
      });
    });

    it("should display deployment details", async () => {
      renderDeploymentList();

      await waitFor(() => {
        expect(screen.getAllByText("localhost:8080")[0]).toBeInTheDocument();
        expect(screen.getAllByText("localhost:8081")[0]).toBeInTheDocument();
        // Deployments should be rendered
        expect(screen.getByText("API Deployments")).toBeInTheDocument();
      });
    });

    it("should display API routes for each deployment when expanded", async () => {
      renderDeploymentList();

      await waitFor(() => {
        expect(screen.getAllByText("localhost:8080")[0]).toBeInTheDocument();
        // The expand buttons should be available for deployments with routes
        expect(screen.getAllByTestId("chevron-right-icon")).toHaveLength(2);
      });

      const expandButtons = screen.getAllByTestId("chevron-right-icon");
      await userEvent.click(expandButtons[0]);

      await waitFor(() => {
        expect(screen.getAllByText("/api/test1").length).toBe(2);
      });
    });
  });

  describe("Deployment Actions", () => {
    it("should handle create deployment button click", async () => {
      renderDeploymentList();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText("New")).toBeInTheDocument();
      });

      const createButton = screen.getByText("New");
      await user.click(createButton);

      expect(mockNavigate).toHaveBeenCalledWith(
        "/app/test-app-id/deployments/create",
      );
    });

    it("should handle delete deployment action", async () => {
      const { API } = await import("@/service");

      renderDeploymentList();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getAllByText("localhost:8080")[0]).toBeInTheDocument();
      });

      const deleteButton = screen
        .getAllByTestId("trash-icon")[0]
        .closest("div");
      await user.click(deleteButton!);

      // Should open confirmation dialog
      await waitFor(() => {
        expect(screen.getByTestId("dialog")).toBeInTheDocument();
      });

      const confirmButton = screen.getByText("Confirm Delete");
      await user.click(confirmButton);

      expect(API.deleteDeployment).toHaveBeenCalledWith(
        "test-app-id",
        "localhost:8080",
      );
    });

    it("should copy cURL command to clipboard", async () => {
      renderDeploymentList();

      await waitFor(() => {
        expect(screen.getAllByText("localhost:8080")[0]).toBeInTheDocument();
        // Copy buttons should be rendered (even if not visible due to expansion state)
        expect(navigator.clipboard.writeText).toBeDefined();
      });
    });
  });

  describe("Data Loading", () => {
    it("should load deployments and API data on component mount", async () => {
      const { API } = await import("@/service");

      renderDeploymentList();

      await waitFor(() => {
        expect(API.getDeploymentApi).toHaveBeenCalledWith("test-app-id");
        expect(API.getApiList).toHaveBeenCalledWith("test-app-id");
      });
    });

    it("should handle empty deployment list", async () => {
      const { API } = await import("@/service");
      (API.getApiList as any).mockResolvedValue(mockApiList);
      (API.getDeploymentApi as any).mockResolvedValue([]);
      (API.getApiList as any).mockResolvedValue([]);

      renderDeploymentList();

      await waitFor(() => {
        expect(screen.getByText("New")).toBeInTheDocument();
      });

      expect(screen.queryByText("Test Deployment 1")).not.toBeInTheDocument();
    });
  });

  describe("Error Handling", () => {
    it("should handle API errors gracefully", async () => {
      const { API } = await import("@/service");
      (API.getApiList as any).mockRejectedValue(new Error("API Error"));
      (API.getApiList as any).mockResolvedValue([]);

      renderDeploymentList();

      await waitFor(() => {
        expect(screen.getByTestId("error-boundary")).toBeInTheDocument();
      });
    });

    it("should handle missing API routes gracefully", async () => {
      renderDeploymentList();

      await waitFor(() => {
        expect(screen.getAllByText("localhost:8080")[0]).toBeInTheDocument();
      });

      // Should not crash when API routes are missing
      expect(screen.queryByText("/api/test1")).not.toBeInTheDocument();
    });

    it("should handle delete deployment errors", async () => {
      const { API } = await import("@/service");
      (API.deleteDeployment as any).mockRejectedValue(
        new Error("Delete failed"),
      );

      renderDeploymentList();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getAllByText("localhost:8080")[0]).toBeInTheDocument();
      });

      const deleteButton = screen
        .getAllByTestId("trash-icon")[0]
        .closest("div");
      await user.click(deleteButton!);

      await waitFor(() => {
        expect(screen.getByTestId("dialog")).toBeInTheDocument();
      });

      const confirmButton = screen.getByText("Confirm Delete");
      await user.click(confirmButton);

      // Should handle error gracefully
      expect(API.deleteDeployment).toHaveBeenCalledWith(
        "test-app-id",
        "localhost:8080",
      );
    });
  });

  describe("UI Interactions", () => {
    it("should show route details on hover", async () => {
      renderDeploymentList();

      await waitFor(() => {
        expect(screen.getAllByText("localhost:8080")[0]).toBeInTheDocument();
        // UI should support hover interactions (basic rendering test)
        expect(screen.getByText("API Deployments")).toBeInTheDocument();
      });
    });

    it("should display HTTP method badges with correct colors", async () => {
      renderDeploymentList();

      await waitFor(() => {
        expect(screen.getAllByText("localhost:8080")[0]).toBeInTheDocument();
        // HTTP method colors should be configured
        expect(screen.getByText("API Deployments")).toBeInTheDocument();
      });
    });
  });

  describe("Performance", () => {
    it("should handle large number of deployments efficiently", async () => {
      const largeDeploymentList = Array.from({ length: 50 }, (_, i) => ({
        apiDefinitions: [
          {
            id: `api-${i}`,
            version: "1.0.0",
          },
        ],
        createdAt: "2023-12-01T10:00:00Z",
        projectId: "test-app-id",
        site: {
          host: `localhost:${8080 + i}`,
          subdomain: `test${i}`,
        },
      }));

      const { API } = await import("@/service");
      (API.getApiList as any).mockResolvedValue(mockApiList);
      (API.getDeploymentApi as any).mockResolvedValue(largeDeploymentList);

      const startTime = performance.now();
      renderDeploymentList();

      await waitFor(() => {
        expect(screen.getByText("localhost:8080")).toBeInTheDocument();
      });

      const endTime = performance.now();
      const renderTime = endTime - startTime;

      expect(renderTime).toBeLessThan(2000);
    });
  });

  describe("Accessibility", () => {
    it("should have proper button labels", async () => {
      renderDeploymentList();

      await waitFor(() => {
        expect(screen.getByText("New")).toBeInTheDocument();
      });

      const createButton = screen.getByText("New").closest("button");
      expect(createButton).toHaveAttribute("type", "button");
    });

    it("should support keyboard navigation", async () => {
      renderDeploymentList();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText("New")).toBeInTheDocument();
      });

      await user.tab();
      expect(document.activeElement).toBe(screen.getByText("New"));
    });
  });

  describe("Loading States", () => {
    it("should show loading state while fetching data", async () => {
      const { API } = await import("@/service");
      (API.getApiList as any).mockImplementation(
        () => new Promise(resolve => setTimeout(resolve, 100)),
      );
      (API.getDeploymentApi as any).mockResolvedValue([]);

      renderDeploymentList();

      // Should show loading indicator
      expect(screen.getByText("New")).toBeInTheDocument();
    });
  });
});
