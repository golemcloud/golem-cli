import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { CreateApplication } from "../index";

// Mock dependencies
const mockNavigate = vi.fn();
vi.mock("react-router-dom", async () => {
  const actual = await vi.importActual("react-router-dom");
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

vi.mock("@/hooks/use-toast", () => ({
  toast: vi.fn(),
}));

vi.mock("@/lib/settings", () => ({
  settingsService: {
    addApp: vi.fn(),
  },
}));

vi.mock("@/components/ui/button", () => ({
  Button: ({ children, onClick, disabled, variant, className }: any) => (
    <button
      onClick={onClick}
      disabled={disabled}
      className={className}
      data-variant={variant}
    >
      {children}
    </button>
  ),
}));

vi.mock("@/components/ui/card", () => ({
  Card: ({ children, className }: any) => (
    <div className={className} data-testid="card">
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

vi.mock("@/components/ui/label", () => ({
  Label: ({ children, htmlFor }: any) => (
    <label htmlFor={htmlFor}>{children}</label>
  ),
}));

vi.mock("@/components/ui/select", () => ({
  Select: ({ children, value, onValueChange }: any) => (
    <div>
      <select
        id="language"
        role="combobox"
        value={value}
        onChange={e => onValueChange(e.target.value)}
      >
        <option value="" disabled hidden>
          Select a language
        </option>
        {children}
      </select>
    </div>
  ),
  SelectContent: ({ children }: any) => <>{children}</>,
  SelectItem: ({ children, value }: any) => (
    <option value={value}>{children}</option>
  ),
  SelectTrigger: () => null,
  SelectValue: () => null,
}));

vi.mock("@/components/ui/tooltip", () => ({
  TooltipProvider: ({ children }: any) => <div>{children}</div>,
  Tooltip: ({ children }: any) => <div>{children}</div>,
  TooltipTrigger: ({ children }: any) => <div>{children}</div>,
  TooltipContent: ({ children }: any) => <div>{children}</div>,
}));

vi.mock("lucide-react", () => ({
  FolderOpen: () => <span data-testid="folder-open-icon">📂</span>,
  Info: () => <span data-testid="info-icon">ℹ️</span>,
  ArrowLeft: () => <span data-testid="arrow-left-icon">←</span>,
  Sparkles: () => <span data-testid="sparkles-icon">✨</span>,
}));

describe("CreateApplication", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  const renderCreateApplication = () => {
    return render(
      <MemoryRouter>
        <CreateApplication />
      </MemoryRouter>,
    );
  };

  describe("Component Rendering", () => {
    it("should render the create application form", () => {
      renderCreateApplication();

      expect(screen.getByText("Create New Application")).toBeInTheDocument();
      expect(screen.getByLabelText("Application Name")).toBeInTheDocument();
      expect(screen.getByLabelText("Programming Language")).toBeInTheDocument();
      expect(screen.getByLabelText("Root Folder")).toBeInTheDocument();
      expect(screen.getByText("Create Application")).toBeInTheDocument();
    });

    it("should render back button", () => {
      renderCreateApplication();

      expect(screen.getByText("Back")).toBeInTheDocument();
    });

    it("should render language options", () => {
      renderCreateApplication();

      expect(screen.getByText("Select a language")).toBeInTheDocument();
    });

    it("should render folder selection button", () => {
      renderCreateApplication();

      expect(screen.getByText("Browse")).toBeInTheDocument();
    });
  });

  describe("Form Validation", () => {
    it("should show validation error for empty app name", async () => {
      renderCreateApplication();
      const user = userEvent.setup();

      const createButton = screen.getByText("Create Application");
      await user.click(createButton);

      await waitFor(() => {
        expect(
          screen.getByText("Application name is required"),
        ).toBeInTheDocument();
      });
    });

    it("should show validation error for invalid app name characters", async () => {
      renderCreateApplication();
      const user = userEvent.setup();

      const nameInput = screen.getByLabelText("Application Name");
      await user.type(nameInput, "invalid name!");

      const createButton = screen.getByText("Create Application");
      await user.click(createButton);

      await waitFor(() => {
        expect(
          screen.getByText(
            "Application name can only contain alphanumeric characters, hyphens, and underscores",
          ),
        ).toBeInTheDocument();
      });
    });

    it("should show validation error for empty folder path", async () => {
      renderCreateApplication();
      const user = userEvent.setup();

      const nameInput = screen.getByLabelText("Application Name");
      await user.type(nameInput, "valid-name");

      const createButton = screen.getByText("Create Application");
      await user.click(createButton);

      await waitFor(() => {
        expect(screen.getByText("Root folder is required")).toBeInTheDocument();
      });
    });

    it("should show validation error for empty language selection", async () => {
      const { toast } = await import("@/hooks/use-toast");

      renderCreateApplication();
      const user = userEvent.setup();

      const nameInput = screen.getByLabelText("Application Name");
      await user.type(nameInput, "valid-name");

      const createButton = screen.getByText("Create Application");
      await user.click(createButton);

      await waitFor(() => {
        expect(toast).toHaveBeenCalledWith({
          title: "Please select a programming language",
          variant: "destructive",
        });
      });
    });
  });

  describe("Form Interactions", () => {
    it("should handle back button click", async () => {
      renderCreateApplication();
      const user = userEvent.setup();

      const backButton = screen.getByText("Back");
      await user.click(backButton);

      expect(mockNavigate).toHaveBeenCalledWith("/");
    });

    it("should handle folder selection", async () => {
      const { open } = await import("@tauri-apps/plugin-dialog");
      (open as any).mockResolvedValue("/path/to/folder");

      renderCreateApplication();
      const user = userEvent.setup();

      const selectFolderButton = screen.getByText("Browse");
      await user.click(selectFolderButton);

      await waitFor(() => {
        expect(open).toHaveBeenCalledWith({
          directory: true,
          multiple: false,
          title: "Select root folder",
        });
      });
    });

    it("should handle language selection", async () => {
      renderCreateApplication();
      const user = userEvent.setup();

      const languageSelect = screen.getByRole("combobox");
      await user.selectOptions(languageSelect, "rust");

      expect(languageSelect).toHaveValue("rust");
    });

    it("should handle form submission with valid data", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { open } = await import("@tauri-apps/plugin-dialog");
      const { settingsService } = await import("@/lib/settings");
      const { toast } = await import("@/hooks/use-toast");

      (open as any).mockResolvedValue("/path/to/folder");
      (invoke as any).mockResolvedValue("app-id-123");
      (settingsService.addApp as any).mockResolvedValue(true);

      renderCreateApplication();
      const user = userEvent.setup();

      // Fill form
      const nameInput = screen.getByLabelText("Application Name");
      await user.type(nameInput, "test-app");

      const selectFolderButton = screen.getByText("Browse");
      await user.click(selectFolderButton);

      const languageSelect = screen.getByRole("combobox");
      await user.selectOptions(languageSelect, "rust");

      // Submit form
      const createButton = screen.getByText("Create Application");
      await user.click(createButton);

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith("create_golem_app", {
          folderPath: "/path/to/folder",
          appName: "test-app",
          language: "rust",
        });
        expect(toast).toHaveBeenCalledWith({
          title: "Application created successfully",
          description: "app-id-123",
        });
        expect(mockNavigate).toHaveBeenCalledWith("/");
      });
    });
  });

  describe("Error Handling", () => {
    it("should handle creation failure", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { open } = await import("@tauri-apps/plugin-dialog");
      const { toast } = await import("@/hooks/use-toast");

      (open as any).mockResolvedValue("/path/to/folder");
      (invoke as any).mockRejectedValue(new Error("Creation failed"));

      renderCreateApplication();
      const user = userEvent.setup();

      // Fill form
      const nameInput = screen.getByLabelText("Application Name");
      await user.type(nameInput, "test-app");

      const selectFolderButton = screen.getByText("Browse");
      await user.click(selectFolderButton);

      const languageSelect = screen.getByRole("combobox");
      await user.selectOptions(languageSelect, "rust");

      // Submit form
      const createButton = screen.getByText("Create Application");
      await user.click(createButton);

      await waitFor(() => {
        expect(toast).toHaveBeenCalledWith({
          title: "Error creating application",
          description: "Error: Creation failed",
          variant: "destructive",
        });
      });
    });

    it("should handle folder selection cancellation", async () => {
      const { open } = await import("@tauri-apps/plugin-dialog");
      (open as any).mockResolvedValue(null);

      renderCreateApplication();
      const user = userEvent.setup();

      const selectFolderButton = screen.getByText("Browse");
      await user.click(selectFolderButton);

      await waitFor(() => {
        expect(open).toHaveBeenCalled();
      });

      // Should not show any error for cancellation
      expect(screen.queryByText(/error/i)).not.toBeInTheDocument();
    });
  });

  describe("Loading States", () => {
    it("should show loading state during creation", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { open } = await import("@tauri-apps/plugin-dialog");

      (open as any).mockResolvedValue("/path/to/folder");
      (invoke as any).mockImplementation(
        () => new Promise(resolve => setTimeout(resolve, 100)),
      );

      renderCreateApplication();
      const user = userEvent.setup();

      // Fill form
      const nameInput = screen.getByLabelText("Application Name");
      await user.type(nameInput, "test-app");

      const selectFolderButton = screen.getByText("Browse");
      await user.click(selectFolderButton);

      const languageSelect = screen.getByRole("combobox");
      await user.selectOptions(languageSelect, "rust");

      // Submit form
      const createButton = screen.getByText("Create Application");
      await user.click(createButton);

      // Check loading state
      expect(createButton).toBeDisabled();
      expect(screen.getByText("Creating Application...")).toBeInTheDocument();
    });
  });

  describe("Accessibility", () => {
    it("should have proper form labels and structure", () => {
      renderCreateApplication();

      expect(screen.getByLabelText("Application Name")).toBeInTheDocument();
      expect(screen.getByLabelText("Programming Language")).toBeInTheDocument();
      expect(screen.getByLabelText("Root Folder")).toBeInTheDocument();
    });

    it("should have proper heading structure", () => {
      renderCreateApplication();

      expect(
        screen.getByRole("heading", { name: "Create New Application" }),
      ).toBeInTheDocument();
    });
  });
});
