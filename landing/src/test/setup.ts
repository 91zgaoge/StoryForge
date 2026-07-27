import "@testing-library/jest-dom/vitest";
import { afterEach, vi } from "vitest";

class IntersectionObserverMock {
  observe = vi.fn();
  disconnect = vi.fn();
  unobserve = vi.fn();
}

Object.defineProperty(window, "IntersectionObserver", {
  writable: true,
  configurable: true,
  value: IntersectionObserverMock,
});

Object.defineProperty(window, "matchMedia", {
  writable: true,
  configurable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// Restore any vi.spyOn mocks (e.g. global.fetch) between tests so a fetch spy
// in one test file never leaks into another. vi.stubGlobal is cleaned up by the
// per-test afterEach in the suites that use it.
afterEach(() => {
  vi.restoreAllMocks();
});
