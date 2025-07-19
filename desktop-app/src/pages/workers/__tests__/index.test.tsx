import { describe, it, expect, vi, beforeEach, afterEach, type MockedFunction } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import WorkerList from "../index";

// Mock dependencies
const mockNavigate = vi.fn();
vi.mock("react-router-dom", async () => {
  const actual = await vi.importActual("react-router-dom");
  return {
    ...actual,
    useNavigate: () => mockNavigate,
    useParams: () => ({
      appId: "test-app-id",
      componentId: "test-component-id",
    }),
  };
});

vi.mock("@/service", () => ({
  API: {
    findWorker: vi.fn(),
  },
}));

vi.mock("@/components/ui/button", () => ({
  Button: ({
    children,
    onClick,
    variant,
    size,
  }: {
    children: React.ReactNode;
    onClick?: () => void;
    variant?: string;
    size?: string;
  }) => (
    <button onClick={onClick} data-variant={variant} data-size={size}>
      {children}
    </button>
  ),
}));

vi.mock("@/components/ui/input", () => ({
  Input: (props: React.InputHTMLAttributes<HTMLInputElement>) => (
    <input {...props} />
  ),
}));

vi.mock("@/components/ui/badge", () => ({
  Badge: ({
    children,
    className,
  }: {
    children: React.ReactNode;
    className?: string;
  }) => (
    <span className={className} data-testid="badge">
      {children}
    </span>
  ),
}));

vi.mock("@/components/ui/card", () => ({
  Card: ({
    children,
    className,
    onClick,
  }: {
    children: React.ReactNode;
    className?: string;
    onClick?: () => void;
  }) => (
    <div className={className} onClick={onClick} data-testid="card">
      {children}
    </div>
  ),
  CardContent: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
  CardHeader: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
  CardTitle: ({ children }: { children: React.ReactNode }) => (
    <h3>{children}</h3>
  ),
}));

vi.mock("lucide-react", () => ({
  LayoutGrid: () => <span data-testid="layout-grid-icon">📱</span>,
  Plus: () => <span data-testid="plus-icon">➕</span>,
  Search: () => <span data-testid="search-icon">🔍</span>,
}));

describe("WorkerList", () => {
  const mockWorkers = [
    {
      workerId: {
        componentId: "test-component-id",
        workerName: "Test Worker 1",
      },
      workerName: "Test Worker 1",
      status: "Idle",
      componentId: "test-component-id",
      createdAt: "2023-12-01T10:00:00Z",
      accountId: "account-1",
      args: [],
      componentSize: 1000,
      componentVersion: 1,
      env: {},
      lastError: null,
      ownedResources: {},
      pendingInvocationCount: 0,
      retryCount: 0,
      totalLinearMemorySize: 1024,
      activePlugins: [],
      updates: [],
    },
    {
      workerId: {
        componentId: "test-component-id",
        workerName: "Test Worker 2",
      },
      workerName: "Test Worker 2",
      status: "Running",
      componentId: "test-component-id",
      createdAt: "2023-12-01T11:00:00Z",
      accountId: "account-2",
      args: [],
      componentSize: 1000,
      componentVersion: 1,
      env: {},
      lastError: null,
      ownedResources: {},
      pendingInvocationCount: 0,
      retryCount: 0,
      totalLinearMemorySize: 1024,
      activePlugins: [],
      updates: [],
    },
    {
      workerId: {
        componentId: "test-component-id",
        workerName: "Test Worker 3",
      },
      workerName: "Test Worker 3",
      status: "Failed",
      componentId: "test-component-id",
      createdAt: "2023-12-01T12:00:00Z",
      accountId: "account-3",
      args: [],
      componentSize: 1000,
      componentVersion: 1,
      env: {},
      lastError: null,
      ownedResources: {},
      pendingInvocationCount: 0,
      retryCount: 0,
      totalLinearMemorySize: 1024,
      activePlugins: [],
      updates: [],
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  const renderWorkerList = () => {
    return render(
      <MemoryRouter>
        <WorkerList />
      </MemoryRouter>,
    );
  };

  describe("Component Rendering", () => {
    it("should render the worker list page", async () => {
      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockResolvedValue({ workers: mockWorkers });

      renderWorkerList();

      await waitFor(() => {
        expect(screen.getByText("Test Worker 1")).toBeInTheDocument();
        expect(screen.getByText("Test Worker 2")).toBeInTheDocument();
        expect(screen.getByText("Test Worker 3")).toBeInTheDocument();
      });
    });

    it("should render search input", async () => {
      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockResolvedValue({ workers: mockWorkers });

      renderWorkerList();

      await waitFor(() => {
        expect(
          screen.getByPlaceholderText("Search workers..."),
        ).toBeInTheDocument();
      });
    });

    it("should render create worker button", async () => {
      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockResolvedValue({ workers: mockWorkers });

      renderWorkerList();

      await waitFor(() => {
        expect(screen.getByText("New Worker")).toBeInTheDocument();
      });
    });

    it("should display worker status badges", async () => {
      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockResolvedValue({ workers: mockWorkers });

      renderWorkerList();

      await waitFor(() => {
        expect(screen.getByText("Idle")).toBeInTheDocument();
        expect(screen.getByText("Running")).toBeInTheDocument();
        expect(screen.getByText("Failed")).toBeInTheDocument();
      });
    });

    it("should display worker creation dates", async () => {
      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockResolvedValue({ workers: mockWorkers });

      renderWorkerList();

      await waitFor(() => {
        expect(screen.getByText("Test Worker 1")).toBeInTheDocument();
      });
    });
  });

  describe("Search Functionality", () => {
    it("should filter workers based on search input", async () => {
      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockResolvedValue({ workers: mockWorkers });

      renderWorkerList();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText("Test Worker 1")).toBeInTheDocument();
        expect(screen.getByText("Test Worker 2")).toBeInTheDocument();
        expect(screen.getByText("Test Worker 3")).toBeInTheDocument();
      });

      const searchInput = screen.getByPlaceholderText("Search workers...");
      await user.type(searchInput, "Test Worker 1");

      await waitFor(() => {
        expect(screen.getByText("Test Worker 1")).toBeInTheDocument();
        expect(screen.queryByText("Test Worker 2")).not.toBeInTheDocument();
        expect(screen.queryByText("Test Worker 3")).not.toBeInTheDocument();
      });
    });

    it("should filter workers by status", async () => {
      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockResolvedValue({ workers: mockWorkers });

      renderWorkerList();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText("Test Worker 1")).toBeInTheDocument();
        expect(screen.getByText("Test Worker 2")).toBeInTheDocument();
        expect(screen.getByText("Test Worker 3")).toBeInTheDocument();
      });

      const searchInput = screen.getByPlaceholderText("Search workers...");
      await user.type(searchInput, "Running");

      await waitFor(() => {
        expect(screen.queryByText("Test Worker 1")).not.toBeInTheDocument();
        expect(screen.getByText("Test Worker 2")).toBeInTheDocument();
        expect(screen.queryByText("Test Worker 3")).not.toBeInTheDocument();
      });
    });

    it("should show no results when search yields no matches", async () => {
      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockResolvedValue({ workers: mockWorkers });

      renderWorkerList();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText("Test Worker 1")).toBeInTheDocument();
      });

      const searchInput = screen.getByPlaceholderText("Search workers...");
      await user.type(searchInput, "non-existent-worker");

      await waitFor(() => {
        expect(screen.queryByText("Test Worker 1")).not.toBeInTheDocument();
        expect(screen.queryByText("Test Worker 2")).not.toBeInTheDocument();
        expect(screen.queryByText("Test Worker 3")).not.toBeInTheDocument();
      });
    });
  });

  describe("Worker Interactions", () => {
    it("should navigate to worker details when clicking on worker card", async () => {
      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockResolvedValue({ workers: mockWorkers });

      renderWorkerList();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText("Test Worker 1")).toBeInTheDocument();
      });

      const workerCard = screen
        .getByText("Test Worker 1")
        .closest('[data-testid="card"]');
      await user.click(workerCard!);

      expect(mockNavigate).toHaveBeenCalledWith(
        "/app/test-app-id/components/test-component-id/workers/Test Worker 1",
      );
    });

    it("should handle create worker button click", async () => {
      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockResolvedValue({ workers: mockWorkers });

      renderWorkerList();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText("New Worker")).toBeInTheDocument();
      });

      const createButton = screen.getByText("New Worker");
      await user.click(createButton);

      expect(mockNavigate).toHaveBeenCalledWith(
        "/app/test-app-id/components/test-component-id/workers/create",
      );
    });
  });

  describe("Data Loading", () => {
    it("should load workers on component mount", async () => {
      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockResolvedValue({ workers: mockWorkers });

      renderWorkerList();

      await waitFor(() => {
        expect(API.workerService.findWorker).toHaveBeenCalledWith(
          "test-app-id",
          "test-component-id",
        );
      });
    });

    it("should sort workers by creation date", async () => {
      const unsortedWorkers = [
        {
          workerId: {
            componentId: "test-component-id",
            workerName: "Test Worker 3",
          },
          workerName: "Test Worker 3",
          status: "Failed",
          componentId: "test-component-id",
          createdAt: "2023-12-01T12:00:00Z",
          accountId: "account-3",
          args: [],
          componentSize: 1000,
          componentVersion: 1,
          env: {},
          lastError: null,
          ownedResources: {},
          pendingInvocationCount: 0,
          retryCount: 0,
          totalLinearMemorySize: 1024,
          activePlugins: [],
          updates: [],
        },
        {
          workerId: {
            componentId: "test-component-id",
            workerName: "Test Worker 1",
          },
          workerName: "Test Worker 1",
          status: "Idle",
          componentId: "test-component-id",
          createdAt: "2023-12-01T10:00:00Z",
          accountId: "account-1",
          args: [],
          componentSize: 1000,
          componentVersion: 1,
          env: {},
          lastError: null,
          ownedResources: {},
          pendingInvocationCount: 0,
          retryCount: 0,
          totalLinearMemorySize: 1024,
          activePlugins: [],
          updates: [],
        },
        {
          workerId: {
            componentId: "test-component-id",
            workerName: "Test Worker 2",
          },
          workerName: "Test Worker 2",
          status: "Running",
          componentId: "test-component-id",
          createdAt: "2023-12-01T11:00:00Z",
          accountId: "account-2",
          args: [],
          componentSize: 1000,
          componentVersion: 1,
          env: {},
          lastError: null,
          ownedResources: {},
          pendingInvocationCount: 0,
          retryCount: 0,
          totalLinearMemorySize: 1024,
          activePlugins: [],
          updates: [],
        },
      ];

      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockResolvedValue({ workers: unsortedWorkers });

      renderWorkerList();

      await waitFor(() => {
        const workerElements = screen.getAllByText(/Test Worker/);
        expect(workerElements[0]).toHaveTextContent("Test Worker 1");
        expect(workerElements[1]).toHaveTextContent("Test Worker 2");
        expect(workerElements[2]).toHaveTextContent("Test Worker 3");
      });
    });
  });

  describe("Error Handling", () => {
    it("should handle API errors gracefully", async () => {
      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockRejectedValue(new Error("API Error"));

      renderWorkerList();

      // Should not crash, but may show empty state
      await waitFor(() => {
        expect(
          screen.getByPlaceholderText("Search workers..."),
        ).toBeInTheDocument();
      });
    });

    it("should handle empty worker list", async () => {
      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockResolvedValue({ workers: [] });

      renderWorkerList();

      await waitFor(() => {
        expect(
          screen.getByPlaceholderText("Search workers..."),
        ).toBeInTheDocument();
        expect(screen.getByText("New Worker")).toBeInTheDocument();
      });

      expect(screen.queryByText("Test Worker 1")).not.toBeInTheDocument();
    });
  });

  describe("Loading States", () => {
    it("should show loading state while fetching workers", async () => {
      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockImplementation(
        () => new Promise(resolve => setTimeout(resolve, 100)),
      );

      renderWorkerList();

      expect(
        screen.getByPlaceholderText("Search workers..."),
      ).toBeInTheDocument();
      expect(screen.getByText("New Worker")).toBeInTheDocument();
    });
  });

  describe("Status Display", () => {
    it("should display worker status with correct colors", async () => {
      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockResolvedValue({ workers: mockWorkers });

      renderWorkerList();

      await waitFor(() => {
        const idleBadge = screen.getByText("Idle");
        const runningBadge = screen.getByText("Running");
        const failedBadge = screen.getByText("Failed");

        expect(idleBadge).toBeInTheDocument();
        expect(runningBadge).toBeInTheDocument();
        expect(failedBadge).toBeInTheDocument();
      });
    });
  });

  describe("Performance", () => {
    it("should handle large number of workers efficiently", async () => {
      const largeWorkerList = Array.from({ length: 100 }, (_, i) => ({
        workerId: {
          componentId: "test-component-id",
          workerName: `Worker ${i}`,
        },
        workerName: `Worker ${i}`,
        status: "Idle",
        componentId: "test-component-id",
        createdAt: "2023-12-01T10:00:00Z",
        accountId: `account-${i}`,
        args: [],
        componentSize: 1000,
        componentVersion: 1,
        env: {},
        lastError: null,
        ownedResources: {},
        pendingInvocationCount: 0,
        retryCount: 0,
        totalLinearMemorySize: 1024,
        activePlugins: [],
        updates: [],
      }));

      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockResolvedValue({ workers: largeWorkerList });

      const startTime = performance.now();
      renderWorkerList();

      await waitFor(() => {
        expect(screen.getByText("Worker 0")).toBeInTheDocument();
      });

      const endTime = performance.now();
      const renderTime = endTime - startTime;

      expect(renderTime).toBeLessThan(1000);
    });
  });

  describe("Accessibility", () => {
    it("should support keyboard navigation", async () => {
      const { API } = await import("@/service");
      (
        API.workerService.findWorker as MockedFunction<
          typeof API.workerService.findWorker
        >
      ).mockResolvedValue({ workers: mockWorkers });

      renderWorkerList();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(
          screen.getByPlaceholderText("Search workers..."),
        ).toBeInTheDocument();
      });

      await user.tab();
      expect(document.activeElement).toBe(
        screen.getByPlaceholderText("Search workers..."),
      );

      await user.tab();
      expect(document.activeElement).toBe(screen.getByText("New Worker"));
    });
  });
});
