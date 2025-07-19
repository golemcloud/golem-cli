import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { SettingsPage } from "../index";

// Mock dependencies
vi.mock("@/components/ui/card", () => ({
  Card: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="card">{children}</div>
  ),
  CardContent: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
  CardDescription: ({ children }: { children: React.ReactNode }) => (
    <p>{children}</p>
  ),
  CardHeader: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
  CardTitle: ({ children }: { children: React.ReactNode }) => (
    <h2>{children}</h2>
  ),
}));

vi.mock("@/components/golem-cli-path", () => ({
  GolemCliPathSetting: () => (
    <div data-testid="golem-cli-path-setting">Golem CLI Path Setting</div>
  ),
}));

describe("SettingsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  const renderSettingsPage = () => {
    return render(
      <MemoryRouter>
        <SettingsPage />
      </MemoryRouter>,
    );
  };

  describe("Component Rendering", () => {
    it("should render the settings page", () => {
      renderSettingsPage();

      expect(screen.getByText("Settings")).toBeInTheDocument();
    });

    it("should render the main heading", () => {
      renderSettingsPage();

      expect(
        screen.getByRole("heading", { name: "Settings" }),
      ).toBeInTheDocument();
    });

    it("should render the Golem CLI Path card", () => {
      renderSettingsPage();

      expect(screen.getByText("Golem CLI Path")).toBeInTheDocument();
      expect(
        screen.getByText("Configure the path to the golem-cli executable"),
      ).toBeInTheDocument();
    });

    it("should render the Golem CLI Path setting component", () => {
      renderSettingsPage();

      expect(screen.getByTestId("golem-cli-path-setting")).toBeInTheDocument();
    });

    it("should render within a card container", () => {
      renderSettingsPage();

      expect(screen.getByTestId("card")).toBeInTheDocument();
    });
  });

  describe("Layout and Structure", () => {
    it("should have proper container layout", () => {
      renderSettingsPage();

      const container = screen.getByText("Settings").closest(".container");
      expect(container).toHaveClass("container", "mx-auto", "px-4", "py-8");
    });

    it("should have proper spacing between elements", () => {
      renderSettingsPage();

      const flexContainer = screen.getByText("Settings").parentElement;
      expect(flexContainer).toHaveClass("flex", "flex-col", "space-y-8");
    });

    it("should have max width constraint", () => {
      renderSettingsPage();

      const flexContainer = screen.getByText("Settings").parentElement;
      expect(flexContainer).toHaveClass("max-w-2xl", "mx-auto");
    });
  });

  describe("Content Structure", () => {
    it("should display the card title correctly", () => {
      renderSettingsPage();

      expect(screen.getByText("Golem CLI Path")).toBeInTheDocument();
    });

    it("should display the card description correctly", () => {
      renderSettingsPage();

      expect(
        screen.getByText("Configure the path to the golem-cli executable"),
      ).toBeInTheDocument();
    });

    it("should contain the GolemCliPathSetting component", () => {
      renderSettingsPage();

      expect(screen.getByText("Golem CLI Path Setting")).toBeInTheDocument();
    });
  });

  describe("Component Integration", () => {
    it("should properly integrate with the GolemCliPathSetting component", () => {
      renderSettingsPage();

      // The GolemCliPathSetting component should be rendered within the card content
      const card = screen.getByTestId("card");
      expect(card).toContainElement(
        screen.getByTestId("golem-cli-path-setting"),
      );
    });
  });

  describe("Accessibility", () => {
    it("should have proper heading structure", () => {
      renderSettingsPage();

      expect(
        screen.getByRole("heading", { level: 1, name: "Settings" }),
      ).toBeInTheDocument();
      expect(
        screen.getByRole("heading", { level: 2, name: "Golem CLI Path" }),
      ).toBeInTheDocument();
    });

    it("should have proper semantic structure", () => {
      renderSettingsPage();

      const mainHeading = screen.getByRole("heading", { level: 1 });
      const sectionHeading = screen.getByRole("heading", { level: 2 });

      expect(mainHeading).toBeInTheDocument();
      expect(sectionHeading).toBeInTheDocument();
    });
  });

  describe("Responsive Design", () => {
    it("should have responsive container classes", () => {
      renderSettingsPage();

      const container = screen.getByText("Settings").closest(".container");
      expect(container).toHaveClass("container", "mx-auto", "px-4", "py-8");
    });

    it("should have responsive max width", () => {
      renderSettingsPage();

      const flexContainer = screen.getByText("Settings").parentElement;
      expect(flexContainer).toHaveClass("max-w-2xl", "mx-auto");
    });
  });

  describe("Export Functionality", () => {
    it("should export as default", () => {
      expect(SettingsPage).toBeDefined();
    });

    it("should be importable as named export", async () => {
      const { default: DefaultSettingsPage } = await import("../index");
      expect(DefaultSettingsPage).toBeDefined();
    });
  });

  describe("Component Stability", () => {
    it("should render consistently across multiple renders", () => {
      const { unmount } = renderSettingsPage();

      expect(screen.getByText("Settings")).toBeInTheDocument();
      expect(screen.getByText("Golem CLI Path")).toBeInTheDocument();

      unmount();

      renderSettingsPage();

      expect(screen.getByText("Settings")).toBeInTheDocument();
      expect(screen.getByText("Golem CLI Path")).toBeInTheDocument();
    });

    it("should not have any side effects", () => {
      const consoleSpy = vi
        .spyOn(console, "error")
        .mockImplementation(() => {});

      renderSettingsPage();

      expect(consoleSpy).not.toHaveBeenCalled();

      consoleSpy.mockRestore();
    });
  });

  describe("Performance", () => {
    it("should render quickly", () => {
      const startTime = performance.now();
      renderSettingsPage();
      const endTime = performance.now();

      expect(endTime - startTime).toBeLessThan(100);
    });
  });
});
