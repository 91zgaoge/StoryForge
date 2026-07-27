import { renderHook, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  useLatestRelease,
  buildReleaseUrls,
  FALLBACK_RELEASE,
  FALLBACK_VERSION,
  __resetLatestReleaseCache,
} from "../useLatestRelease";

function mockManifestOk(payload: unknown) {
  return vi.spyOn(globalThis, "fetch").mockResolvedValue({
    ok: true,
    status: 200,
    json: async () => payload,
  } as Response);
}

describe("buildReleaseUrls", () => {
  it("builds platform download urls from a version", () => {
    const urls = buildReleaseUrls("1.2.3");
    expect(urls.mac).toBe(
      "https://storymoss.top/releases/StoryMoss_1.2.3_aarch64.dmg",
    );
    expect(urls.windows).toBe(
      "https://storymoss.top/releases/StoryMoss_1.2.3_x64_zh-CN.msi",
    );
    expect(urls.linux).toBe(
      "https://storymoss.top/releases/StoryMoss_1.2.3_amd64.AppImage",
    );
  });
});

describe("useLatestRelease", () => {
  beforeEach(() => {
    __resetLatestReleaseCache();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("returns the manifest version and derived urls", async () => {
    mockManifestOk({ version: "0.30.99" });
    const { result } = renderHook(() => useLatestRelease());
    // Initial render uses the fallback before the fetch resolves.
    expect(result.current.version).toBe(FALLBACK_VERSION);
    await waitFor(() => {
      expect(result.current.version).toBe("0.30.99");
    });
    expect(result.current.urls.mac).toBe(
      "https://storymoss.top/releases/StoryMoss_0.30.99_aarch64.dmg",
    );
    expect(result.current.urls.windows).toBe(
      "https://storymoss.top/releases/StoryMoss_0.30.99_x64_zh-CN.msi",
    );
    expect(result.current.urls.linux).toBe(
      "https://storymoss.top/releases/StoryMoss_0.30.99_amd64.AppImage",
    );
  });

  it("strips a leading v from the manifest version", async () => {
    mockManifestOk({ version: "v0.30.99" });
    const { result } = renderHook(() => useLatestRelease());
    await waitFor(() => {
      expect(result.current.version).toBe("0.30.99");
    });
  });

  it("falls back when fetch rejects", async () => {
    vi.spyOn(globalThis, "fetch").mockRejectedValue(new Error("network"));
    const { result } = renderHook(() => useLatestRelease());
    await waitFor(() => {
      expect(result.current.version).toBe(FALLBACK_VERSION);
    });
    expect(result.current.urls).toEqual(FALLBACK_RELEASE.urls);
  });

  it("falls back when fetch responds non-ok", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue({
      ok: false,
      status: 500,
      json: async () => ({}),
    } as Response);
    const { result } = renderHook(() => useLatestRelease());
    await waitFor(() => {
      expect(result.current.version).toBe(FALLBACK_VERSION);
    });
  });

  it("shares a single fetch across multiple mounts (module cache)", async () => {
    const spy = mockManifestOk({ version: "0.30.99" });
    const r1 = renderHook(() => useLatestRelease());
    const r2 = renderHook(() => useLatestRelease());
    const r3 = renderHook(() => useLatestRelease());
    await waitFor(() => expect(r1.result.current.version).toBe("0.30.99"));
    await waitFor(() => expect(r2.result.current.version).toBe("0.30.99"));
    await waitFor(() => expect(r3.result.current.version).toBe("0.30.99"));
    expect(spy).toHaveBeenCalledTimes(1);
    r1.unmount();
    r2.unmount();
    r3.unmount();
  });
});
