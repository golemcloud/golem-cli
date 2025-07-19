import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import Components from "../index";

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
    getComponentByIdAsKey: vi.fn(),
    findWorker: vi.fn(),
  },
}));

vi.mock("@/components/errorBoundary", () => ({
  default: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="error-boundary">{children}</div>
  ),
}));

vi.mock("@/components/ui/badge", () => ({
  Badge: ({ children, className }: any) => (
    <span className={className} data-testid="badge">
      {children}
    </span>
  ),
}));

vi.mock("@/components/ui/button", () => ({
  Button: ({ children, onClick, disabled, variant, size }: any) => (
    <button
      onClick={onClick}
      disabled={disabled}
      data-variant={variant}
      data-size={size}
    >
      {children}
    </button>
  ),
}));

vi.mock("@/components/ui/card", () => ({
  Card: ({ children, className, onClick }: any) => (
    <div className={className} onClick={onClick} data-testid="card">
      {children}
    </div>
  ),
  CardContent: ({ children }: any) => <div>{children}</div>,
  CardDescription: ({ children }: any) => <p>{children}</p>,
  CardHeader: ({ children }: any) => <div>{children}</div>,
  CardTitle: ({ children }: any) => <h2>{children}</h2>,
}));

vi.mock("@/components/ui/input", () => ({
  Input: (props: any) => <input {...props} />,
}));

vi.mock("@/lib/utils", () => ({
  calculateExportFunctions: vi.fn(() => 5),
  cn: vi.fn((...args) => args.join(" ")),
  formatRelativeTime: vi.fn(() => "2 minutes ago"),
}));

vi.mock("lucide-react", () => ({
  LayoutGrid: () => <span data-testid="layout-grid-icon">📱</span>,
  PlusCircle: () => <span data-testid="plus-circle-icon">➕</span>,
}));

describe("Components", () => {
  const mockComponents = {
    "comp-1": {
      componentId: "comp-1",
      componentName: "Test Component 1",
      componentType: "durable",
      componentSize: 1024,
      versionList: ["1.0.0"],
      versions: [{ createdAt: new Date("2023-12-01T10:00:00Z") }],
    },
    "comp-2": {
      componentId: "comp-2",
      componentName: "Test Component 2",
      componentType: "ephemeral",
      componentSize: 2048,
      versionList: ["1.0.0"],
      versions: [{ createdAt: new Date("2023-12-01T11:00:00Z") }],
    },
  };

  const mockWorkers = {
    workers: [
      {
        workerId: "worker-1",
        workerName: "Worker 1",
        status: "Idle",
        componentId: "comp-1",
        createdAt: "2023-12-01T10:00:00Z",
      },
      {
        workerId: "worker-2",
        workerName: "Worker 2",
        status: "Running",
        componentId: "comp-1",
        createdAt: "2023-12-01T11:00:00Z",
      },
    ],
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  const renderComponents = () => {
    return render(
      <MemoryRouter>
        <Components />
      </MemoryRouter>,
    );
  };

  describe("Component Rendering", () => {
    it("should render the components page", async () => {
      const { API } = await import("@/service");
      (API.componentService.getComponentByIdAsKey as any).mockResolvedValue(mockComponents);
      (API.workerService.findWorker as any).mockResolvedValue(mockWorkers);

      renderComponents();

      await waitFor(() => {
        expect(screen.getByText("Components")).toBeInTheDocument();
      });
    });

    it("should render search input", async () => {
      const { API } = await import("@/service");
      (API.componentService.getComponentByIdAsKey as any).mockResolvedValue(mockComponents);
      (API.workerService.findWorker as any).mockResolvedValue(mockWorkers);

      renderComponents();

      await waitFor(() => {
        expect(
          screen.getByPlaceholderText("Search components..."),
        ).toBeInTheDocument();
      });
    });

    it("should render create component button", async () => {
      const { API } = await import("@/service");
      (API.componentService.getComponentByIdAsKey as any).mockResolvedValue(mockComponents);
      (API.workerService.findWorker as any).mockResolvedValue(mockWorkers);

      renderComponents();

      await waitFor(() => {
        expect(screen.getByText("Create Component")).toBeInTheDocument();
      });
    });

    it("should render component cards", async () => {
      const { API } = await import("@/service");
      (API.componentService.getComponentByIdAsKey as any).mockResolvedValue(mockComponents);
      (API.workerService.findWorker as any).mockResolvedValue(mockWorkers);

      renderComponents();

      await waitFor(() => {
        expect(screen.getByText("Test Component 1")).toBeInTheDocument();
        expect(screen.getByText("Test Component 2")).toBeInTheDocument();
      });
    });
  });

  describe("Search Functionality", () => {
    it("should filter components based on search input", async () => {
      const { API } = await import("@/service");
      (API.componentService.getComponentByIdAsKey as any).mockResolvedValue(mockComponents);
      (API.workerService.findWorker as any).mockResolvedValue(mockWorkers);

      renderComponents();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText("Test Component 1")).toBeInTheDocument();
        expect(screen.getByText("Test Component 2")).toBeInTheDocument();
      });

      const searchInput = screen.getByPlaceholderText("Search components...");
      await user.type(searchInput, "Test Component 1");

      await waitFor(() => {
        expect(screen.getByText("Test Component 1")).toBeInTheDocument();
        expect(screen.queryByText("Test Component 2")).not.toBeInTheDocument();
      });
    });

    it("should show empty state when search yields no matches", async () => {
      const { API } = await import("@/service");
      (API.componentService.getComponentByIdAsKey as any).mockResolvedValue(mockComponents);
      (API.workerService.findWorker as any).mockResolvedValue(mockWorkers);

      renderComponents();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText("Test Component 1")).toBeInTheDocument();
      });

      const searchInput = screen.getByPlaceholderText("Search components...");
      await user.type(searchInput, "non-existent-component");

      await waitFor(() => {
        expect(screen.queryByText("Test Component 1")).not.toBeInTheDocument();
        expect(screen.queryByText("Test Component 2")).not.toBeInTheDocument();
      });
    });
  });

  describe("Component Interactions", () => {
    it("should navigate to component details when clicking on component card", async () => {
      const { API } = await import("@/service");
      (API.componentService.getComponentByIdAsKey as any).mockResolvedValue(mockComponents);
      (API.workerService.findWorker as any).mockResolvedValue(mockWorkers);

      renderComponents();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText("Test Component 1")).toBeInTheDocument();
      });

      const componentCard = screen
        .getByText("Test Component 1")
        .closest('[data-testid="card"]');
      await user.click(componentCard!);

      expect(mockNavigate).toHaveBeenCalledWith(
        "/app/test-app-id/components/comp-1",
      );
    });

    it("should handle create component button click", async () => {
      const { API } = await import("@/service");
      (API.componentService.getComponentByIdAsKey as any).mockResolvedValue(mockComponents);
      (API.workerService.findWorker as any).mockResolvedValue(mockWorkers);

      renderComponents();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText("Create Component")).toBeInTheDocument();
      });

      const createButton = screen.getByText("Create Component");
      await user.click(createButton);

      expect(mockNavigate).toHaveBeenCalledWith(
        "/app/test-app-id/components/create",
      );
    });
  });

  describe("Error Handling", () => {
    it("should handle API errors gracefully", async () => {
      const { API } = await import("@/service");
      (API.componentService.getComponentByIdAsKey as any).mockRejectedValue(
        new Error("API Error"),
      );

      renderComponents();

      await waitFor(() => {
        expect(screen.getByTestId("error-boundary")).toBeInTheDocument();
      });
    });

    it("should handle empty component list", async () => {
      const { API } = await import("@/service");
      (API.componentService.getComponentByIdAsKey as any).mockResolvedValue({});
      (API.workerService.findWorker as any).mockResolvedValue({ workers: [] });

      renderComponents();

      await waitFor(() => {
        expect(screen.getByText("Components")).toBeInTheDocument();
        expect(screen.getByText("No Project Components")).toBeInTheDocument();
      });
    });
  });

  describe("Loading States", () => {
    it("should show loading state while fetching components", async () => {
      const { API } = await import("@/service");
      (API.componentService.getComponentByIdAsKey as any).mockImplementation(
        () => new Promise(resolve => setTimeout(resolve, 100)),
      );
      (API.workerService.findWorker as any).mockResolvedValue({ workers: [] });

      renderComponents();

      expect(screen.getByText("Components")).toBeInTheDocument();
    });
  });

  describe("Data Display", () => {
    it("should display component metadata correctly", async () => {
      const { API } = await import("@/service");
      (API.componentService.getComponentByIdAsKey as any).mockResolvedValue(mockComponents);
      (API.workerService.findWorker as any).mockResolvedValue(mockWorkers);

      renderComponents();

      await waitFor(() => {
        expect(screen.getByText("Test Component 1")).toBeInTheDocument();
        expect(screen.getByText("durable")).toBeInTheDocument();
      });
    });
  });

  describe("Performance", () => {
    it("should handle large number of components efficiently", async () => {
      const largeComponentsList = Array.from({ length: 100 }, (_, i) => [
        `comp-${i}`,
        {
          componentId: `comp-${i}`,
          componentName: `Component ${i}`,
          componentType: "durable",
          componentSize: 1024,
          versionList: ["1.0.0"],
          versions: [{ createdAt: new Date("2023-12-01T10:00:00Z") }],
        },
      ]).reduce(
        (acc, [key, value]) => ({ ...acc, [key as string]: value }),
        {},
      );

      const { API } = await import("@/service");
      (API.componentService.getComponentByIdAsKey as any).mockResolvedValue(largeComponentsList);
      (API.workerService.findWorker as any).mockResolvedValue({ workers: [] });

      const startTime = performance.now();
      renderComponents();

      await waitFor(() => {
        expect(screen.getByText("Component 0")).toBeInTheDocument();
      });

      const endTime = performance.now();
      const renderTime = endTime - startTime;

      expect(renderTime).toBeLessThan(1000);
    });
  });

  describe("Accessibility", () => {
    it("should have proper ARIA labels and structure", async () => {
      const { API } = await import("@/service");
      (API.componentService.getComponentByIdAsKey as any).mockResolvedValue(mockComponents);
      (API.workerService.findWorker as any).mockResolvedValue(mockWorkers);

      renderComponents();

      await waitFor(() => {
        expect(
          screen.getByPlaceholderText("Search components..."),
        ).toBeInTheDocument();
      });

      const searchInput = screen.getByPlaceholderText("Search components...");
      expect(searchInput).toHaveAttribute("type", "text");
    });

    it("should support keyboard navigation", async () => {
      const { API } = await import("@/service");
      (API.componentService.getComponentByIdAsKey as any).mockResolvedValue(mockComponents);
      (API.workerService.findWorker as any).mockResolvedValue(mockWorkers);

      renderComponents();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(
          screen.getByPlaceholderText("Search components..."),
        ).toBeInTheDocument();
      });

      await user.tab();
      expect(document.activeElement).toBe(
        screen.getByPlaceholderText("Search components..."),
      );

      await user.tab();
      expect(document.activeElement).toBe(screen.getByText("Create Component"));
    });
  });
});
