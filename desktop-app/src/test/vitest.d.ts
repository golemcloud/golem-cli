/// <reference types="vitest" />

import type { vi } from "vitest";

declare global {
  const vi: typeof import("vitest").vi;
}

export {};