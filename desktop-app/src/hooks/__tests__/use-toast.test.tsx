import { renderHook, act } from "@testing-library/react";
import { useToast, toast } from "../use-toast";
import { describe, beforeEach, vi, afterEach, it, expect } from "vitest";

describe("useToast", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    // Clear any existing toasts
    const { result } = renderHook(() => useToast());
    act(() => {
      result.current.dismiss();
    });
  });

  afterEach(() => {
    vi.runOnlyPendingTimers();
    vi.useRealTimers();
  });

  it("starts with empty toasts array", () => {
    const { result } = renderHook(() => useToast());
    expect(result.current.toasts).toEqual([]);
  });

  it("adds a toast", () => {
    const { result } = renderHook(() => useToast());

    act(() => {
      result.current.toast({
        title: "Test Toast",
        description: "This is a test toast",
      });
    });

    expect(result.current.toasts).toHaveLength(1);
    expect(result.current.toasts[0].title).toBe("Test Toast");
    expect(result.current.toasts[0].description).toBe("This is a test toast");
    expect(result.current.toasts[0].open).toBe(true);
  });

  it("limits toasts to TOAST_LIMIT", () => {
    const { result } = renderHook(() => useToast());

    act(() => {
      result.current.toast({ title: "Toast 1" });
      result.current.toast({ title: "Toast 2" });
    });

    expect(result.current.toasts).toHaveLength(1);
    expect(result.current.toasts[0].title).toBe("Toast 2");
  });

  it("dismisses a specific toast", () => {
    const { result } = renderHook(() => useToast());

    let toastId: string;
    act(() => {
      const toastInstance = result.current.toast({ title: "Test Toast" });
      toastId = toastInstance.id;
    });

    expect(result.current.toasts).toHaveLength(1);
    expect(result.current.toasts[0].open).toBe(true);

    act(() => {
      result.current.dismiss(toastId!);
    });

    expect(result.current.toasts[0].open).toBe(false);
  });

  it("dismisses all toasts when no ID provided", () => {
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

  it("updates a toast", () => {
    const { result } = renderHook(() => useToast());

    let toastInstance: ReturnType<typeof toast>;
    act(() => {
      toastInstance = result.current.toast({ title: "Original Title" });
    });

    expect(result.current.toasts[0].title).toBe("Original Title");

    act(() => {
      toastInstance.update({ id: toastInstance.id, title: "Updated Title" });
    });

    expect(result.current.toasts[0].title).toBe("Updated Title");
  });

  it("removes toast using dismiss method from toast instance", () => {
    const { result } = renderHook(() => useToast());

    let toastInstance: ReturnType<typeof toast>;
    act(() => {
      toastInstance = result.current.toast({ title: "Test Toast" });
    });

    expect(result.current.toasts).toHaveLength(1);

    act(() => {
      toastInstance.dismiss();
    });

    expect(result.current.toasts[0].open).toBe(false);
  });

  it("handles toast with different variant types", () => {
    const { result } = renderHook(() => useToast());

    act(() => {
      result.current.toast({
        title: "Success Toast",
        variant: "default",
      });
    });

    expect(result.current.toasts[0].variant).toBe("default");
  });

  it("maintains toast state across multiple hook instances", () => {
    const { result: result1 } = renderHook(() => useToast());
    const { result: result2 } = renderHook(() => useToast());

    act(() => {
      result1.current.toast({ title: "Shared Toast" });
    });

    expect(result1.current.toasts).toHaveLength(1);
    expect(result2.current.toasts).toHaveLength(1);
    expect(result2.current.toasts[0].title).toBe("Shared Toast");
  });

  it("calls onOpenChange when toast is dismissed", () => {
    const { result } = renderHook(() => useToast());

    act(() => {
      result.current.toast({ title: "Test Toast" });
    });

    const toast = result.current.toasts[0];
    expect(toast.open).toBe(true);

    act(() => {
      if (toast.onOpenChange) {
        toast.onOpenChange(false);
      }
    });

    expect(result.current.toasts[0].open).toBe(false);
  });
});

describe("toast function", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.runOnlyPendingTimers();
    vi.useRealTimers();
  });

  it("returns toast instance with id, dismiss, and update methods", () => {
    const toastInstance = toast({ title: "Test Toast" });

    expect(toastInstance).toHaveProperty("id");
    expect(toastInstance).toHaveProperty("dismiss");
    expect(toastInstance).toHaveProperty("update");
    expect(typeof toastInstance.id).toBe("string");
    expect(typeof toastInstance.dismiss).toBe("function");
    expect(typeof toastInstance.update).toBe("function");
  });

  it("generates unique IDs for different toasts", () => {
    const toast1 = toast({ title: "Toast 1" });
    const toast2 = toast({ title: "Toast 2" });

    expect(toast1.id).not.toBe(toast2.id);
  });
});
