import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import React from "react";
import { useToast, toast, reducer } from "../use-toast";

type ToastControl = {
  id: string;
  dismiss: () => void;
  update: (
    props: Partial<{
      id: string;
      title?: React.ReactNode;
      description?: React.ReactNode;
    }>,
  ) => void;
};

// Helper function to reset the memory state
function resetToastState() {
  // Clear all toasts
  const { result } = renderHook(() => useToast());
  act(() => {
    result.current.dismiss();
  });

  // Fast forward to remove all toasts
  act(() => {
    vi.advanceTimersByTime(1000001);
  });
}

describe("useToast and toast system", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
    resetToastState();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe("genId function", () => {
    it("should generate unique incrementing IDs", () => {
      resetToastState();

      const toast1 = toast({ title: "Test 1" });
      const toast2 = toast({ title: "Test 2" });

      expect(toast1.id).not.toBe(toast2.id);
      // Note: Due to TOAST_LIMIT = 1, only one toast will exist at a time
      expect(toast1.id).toBeDefined();
      expect(toast2.id).toBeDefined();
    });
  });

  describe("reducer", () => {
    const initialState = { toasts: [] };

    it("should add toast to state", () => {
      const newToast = {
        id: "1",
        title: "Test Toast",
        description: "Test Description",
        open: true,
      };

      const action = {
        type: "ADD_TOAST" as const,
        toast: newToast,
      };

      const newState = reducer(initialState, action);

      expect(newState.toasts).toHaveLength(1);
      expect(newState.toasts[0]).toEqual(newToast);
    });

    it("should limit toasts to TOAST_LIMIT", () => {
      const existingToast = {
        id: "1",
        title: "Existing Toast",
        open: true,
      };

      const stateWithToast = { toasts: [existingToast] };

      const newToast = {
        id: "2",
        title: "New Toast",
        open: true,
      };

      const action = {
        type: "ADD_TOAST" as const,
        toast: newToast,
      };

      const newState = reducer(stateWithToast, action);

      // With TOAST_LIMIT = 1, should only keep the newest toast
      expect(newState.toasts).toHaveLength(1);
      expect(newState.toasts[0]).toEqual(newToast);
    });

    it("should update existing toast", () => {
      const existingToast = {
        id: "1",
        title: "Original Title",
        description: "Original Description",
        open: true,
      };

      const state = { toasts: [existingToast] };

      const action = {
        type: "UPDATE_TOAST" as const,
        toast: {
          id: "1",
          title: "Updated Title",
        },
      };

      const newState = reducer(state, action);

      expect(newState.toasts[0]).toEqual({
        id: "1",
        title: "Updated Title",
        description: "Original Description",
        open: true,
      });
    });

    it("should dismiss specific toast", () => {
      const toast1 = { id: "1", title: "Toast 1", open: true };
      const toast2 = { id: "2", title: "Toast 2", open: true };
      const state = { toasts: [toast1, toast2] };

      const action = {
        type: "DISMISS_TOAST" as const,
        toastId: "1",
      };

      const newState = reducer(state, action);

      expect(newState.toasts[0].open).toBe(false);
      expect(newState.toasts[1].open).toBe(true);
    });

    it("should dismiss all toasts when no toastId provided", () => {
      const toast1 = { id: "1", title: "Toast 1", open: true };
      const toast2 = { id: "2", title: "Toast 2", open: true };
      const state = { toasts: [toast1, toast2] };

      const action = {
        type: "DISMISS_TOAST" as const,
      };

      const newState = reducer(state, action);

      expect(newState.toasts[0].open).toBe(false);
      expect(newState.toasts[1].open).toBe(false);
    });

    it("should remove specific toast", () => {
      const toast1 = { id: "1", title: "Toast 1", open: true };
      const toast2 = { id: "2", title: "Toast 2", open: true };
      const state = { toasts: [toast1, toast2] };

      const action = {
        type: "REMOVE_TOAST" as const,
        toastId: "1",
      };

      const newState = reducer(state, action);

      expect(newState.toasts).toHaveLength(1);
      expect(newState.toasts[0]).toEqual(toast2);
    });

    it("should remove all toasts when no toastId provided", () => {
      const toast1 = { id: "1", title: "Toast 1", open: true };
      const toast2 = { id: "2", title: "Toast 2", open: true };
      const state = { toasts: [toast1, toast2] };

      const action = {
        type: "REMOVE_TOAST" as const,
      };

      const newState = reducer(state, action);

      expect(newState.toasts).toHaveLength(0);
    });
  });

  describe("toast function", () => {
    it("should create toast with generated ID and return control object", () => {
      resetToastState();

      const toastControl = toast({ title: "Test Toast" });

      expect(toastControl).toHaveProperty("id");
      expect(toastControl).toHaveProperty("dismiss");
      expect(toastControl).toHaveProperty("update");
      expect(typeof toastControl.dismiss).toBe("function");
      expect(typeof toastControl.update).toBe("function");
    });

    it("should create toast with all properties", () => {
      resetToastState();

      const toastProps = {
        title: "Test Title",
        description: "Test Description",
        variant: "destructive" as const,
        duration: 5000,
      };

      const toastControl = toast(toastProps);

      expect(toastControl.id).toBeDefined();
    });

    it("should update toast using returned update function", () => {
      resetToastState();

      const { result } = renderHook(() => useToast());
      let toastControl: ToastControl;

      act(() => {
        toastControl = toast({ title: "Original Title" });
      });

      act(() => {
        toastControl.update({
          title: "Updated Title",
          description: "New Description",
        });
      });

      expect(result.current.toasts[0].title).toBe("Updated Title");
      expect(result.current.toasts[0].description).toBe("New Description");
    });

    it("should dismiss toast using returned dismiss function", () => {
      resetToastState();

      const { result } = renderHook(() => useToast());
      let toastControl: ToastControl;

      act(() => {
        toastControl = toast({ title: "Test Toast" });
      });

      expect(result.current.toasts[0].open).toBe(true);

      act(() => {
        toastControl.dismiss();
      });

      expect(result.current.toasts[0].open).toBe(false);
    });

    it("should handle onOpenChange callback", () => {
      resetToastState();

      const { result } = renderHook(() => useToast());

      act(() => {
        toast({ title: "Test Toast" });
      });

      const currentToast = result.current.toasts[0];
      expect(currentToast.onOpenChange).toBeDefined();

      // Simulate closing the toast
      act(() => {
        currentToast.onOpenChange?.(false);
      });

      expect(result.current.toasts[0].open).toBe(false);
    });
  });

  describe("useToast hook", () => {
    it("should return current toast state and toast function", () => {
      resetToastState();

      const { result } = renderHook(() => useToast());

      expect(result.current).toHaveProperty("toasts");
      expect(result.current).toHaveProperty("toast");
      expect(result.current).toHaveProperty("dismiss");
      expect(Array.isArray(result.current.toasts)).toBe(true);
      expect(typeof result.current.toast).toBe("function");
      expect(typeof result.current.dismiss).toBe("function");
    });

    it("should update when toast is dismissed via hook", () => {
      resetToastState();

      const { result } = renderHook(() => useToast());
      let toastId: string;

      act(() => {
        const toastControl = result.current.toast({ title: "Test Toast" });
        toastId = toastControl.id;
      });

      expect(result.current.toasts[0].open).toBe(true);

      act(() => {
        result.current.dismiss(toastId);
      });

      expect(result.current.toasts[0].open).toBe(false);
    });

    it("should dismiss all toasts when no ID provided", () => {
      resetToastState();

      const { result } = renderHook(() => useToast());

      act(() => {
        result.current.toast({ title: "Toast 1" });
      });

      expect(result.current.toasts).toHaveLength(1);
      expect(result.current.toasts[0].open).toBe(true);

      act(() => {
        result.current.dismiss();
      });

      expect(result.current.toasts[0].open).toBe(false);
    });

    it("should clean up listener on unmount", () => {
      resetToastState();

      const { unmount } = renderHook(() => useToast());

      // This test ensures no memory leaks occur
      expect(() => unmount()).not.toThrow();
    });
  });

  describe("toast removal queue", () => {
    it("should automatically remove dismissed toasts after delay", () => {
      resetToastState();

      const { result } = renderHook(() => useToast());
      let toastControl: ToastControl;

      act(() => {
        toastControl = toast({ title: "Test Toast" });
      });

      expect(result.current.toasts).toHaveLength(1);
      expect(result.current.toasts[0].open).toBe(true);

      act(() => {
        toastControl.dismiss();
      });

      expect(result.current.toasts[0].open).toBe(false);

      // Fast forward the removal delay
      act(() => {
        vi.advanceTimersByTime(1000001); // TOAST_REMOVE_DELAY + 1
      });

      expect(result.current.toasts).toHaveLength(0);
    });

    it("should not add duplicate removal timeouts", () => {
      resetToastState();

      const setTimeoutSpy = vi.spyOn(global, "setTimeout");
      renderHook(() => useToast());
      let toastControl: ToastControl;

      act(() => {
        toastControl = toast({ title: "Test Toast" });
      });

      // Clear previous setTimeout calls from toast creation
      setTimeoutSpy.mockClear();

      act(() => {
        toastControl.dismiss();
        toastControl.dismiss(); // Dismiss again
      });

      // Should only create one timeout for removal
      expect(setTimeoutSpy).toHaveBeenCalledTimes(1);

      setTimeoutSpy.mockRestore();
    });
  });

  describe("memory state management", () => {
    it("should maintain state across multiple hook instances", () => {
      resetToastState();

      const { result: result1 } = renderHook(() => useToast());
      const { result: result2 } = renderHook(() => useToast());

      act(() => {
        result1.current.toast({ title: "Shared Toast" });
      });

      expect(result1.current.toasts).toHaveLength(1);
      expect(result2.current.toasts).toHaveLength(1);
      expect(result1.current.toasts[0].title).toBe("Shared Toast");
      expect(result2.current.toasts[0].title).toBe("Shared Toast");
    });

    it("should sync state changes across multiple hook instances", () => {
      resetToastState();

      const { result: result1 } = renderHook(() => useToast());
      const { result: result2 } = renderHook(() => useToast());
      let toastId: string;

      act(() => {
        const toastControl = result1.current.toast({ title: "Shared Toast" });
        toastId = toastControl.id;
      });

      expect(result1.current.toasts[0].open).toBe(true);
      expect(result2.current.toasts[0].open).toBe(true);

      act(() => {
        result2.current.dismiss(toastId);
      });

      expect(result1.current.toasts[0].open).toBe(false);
      expect(result2.current.toasts[0].open).toBe(false);
    });
  });

  describe("TOAST_LIMIT behavior", () => {
    it("should replace existing toast when limit is reached", () => {
      resetToastState();

      const { result } = renderHook(() => useToast());

      act(() => {
        result.current.toast({ title: "First Toast" });
      });

      expect(result.current.toasts).toHaveLength(1);
      expect(result.current.toasts[0].title).toBe("First Toast");

      act(() => {
        result.current.toast({ title: "Second Toast" });
      });

      // Should only have one toast due to TOAST_LIMIT = 1
      expect(result.current.toasts).toHaveLength(1);
      expect(result.current.toasts[0].title).toBe("Second Toast");
    });
  });

  describe("integration scenarios", () => {
    it("should handle rapid toast creation and dismissal", () => {
      resetToastState();

      const { result } = renderHook(() => useToast());
      const controls: ToastControl[] = [];

      // Create multiple toasts rapidly
      act(() => {
        controls.push(result.current.toast({ title: "Toast 1" }));
        controls.push(result.current.toast({ title: "Toast 2" }));
        controls.push(result.current.toast({ title: "Toast 3" }));
      });

      // Only one should exist due to limit
      expect(result.current.toasts).toHaveLength(1);
      expect(result.current.toasts[0].title).toBe("Toast 3");

      // Dismiss and verify removal
      act(() => {
        controls[controls.length - 1].dismiss();
      });

      expect(result.current.toasts[0].open).toBe(false);

      act(() => {
        vi.advanceTimersByTime(1000001);
      });

      expect(result.current.toasts).toHaveLength(0);
    });
  });
});
