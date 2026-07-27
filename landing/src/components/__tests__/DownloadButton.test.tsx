import { render, screen, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  DownloadButton,
  detectPlatform,
  downloadUrl,
  downloadLabel,
} from "../DownloadButton";
import {
  __resetLatestReleaseCache,
  FALLBACK_RELEASE,
} from "../../hooks/useLatestRelease";

function mockManifestOk(payload: unknown) {
  vi.spyOn(globalThis, "fetch").mockResolvedValue({
    ok: true,
    status: 200,
    json: async () => payload,
  } as Response);
}

function mockManifestFail() {
  vi.spyOn(globalThis, "fetch").mockRejectedValue(new Error("network"));
}

describe("DownloadButton", () => {
  beforeEach(() => {
    __resetLatestReleaseCache();
    vi.stubGlobal("navigator", { userAgent: "MacIntel", platform: "MacIntel" });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("renders platform-specific label", async () => {
    mockManifestOk({ version: "0.30.99" });
    render(<DownloadButton variant="primary" />);
    const link = screen.getByRole("link", {
      name: /下载 macOS 版/i,
    });
    expect(link).toBeInTheDocument();
    // Let the async manifest fetch settle inside act() to avoid a pending
    // state update leaking out of the test.
    await waitFor(() => {
      expect(link).toHaveAttribute(
        "href",
        expect.stringContaining("StoryMoss_0.30.99"),
      );
    });
  });

  it("points to the latest release asset from the manifest", async () => {
    // Use a version distinct from the fallback so the dynamic update is real.
    mockManifestOk({ version: "0.30.99" });
    render(<DownloadButton variant="primary" />);
    const link = screen.getByRole("link", {
      name: /下载 macOS 版/i,
    }) as HTMLAnchorElement;
    await waitFor(() => {
      expect(link.href).toContain("StoryMoss_0.30.99");
    });
    expect(link.href).toMatch(/\.dmg$/);
  });

  it("falls back to the bundled fallback version when fetch fails", async () => {
    mockManifestFail();
    render(<DownloadButton variant="primary" />);
    const link = screen.getByRole("link", {
      name: /下载 macOS 版/i,
    }) as HTMLAnchorElement;
    await waitFor(() => {
      expect(link.href).toContain(`StoryMoss_${FALLBACK_RELEASE.version}`);
    });
    expect(link.href).toMatch(/\.dmg$/);
  });

  it("falls back to releases page on unknown platform", async () => {
    mockManifestOk({ version: "0.30.99" });
    vi.stubGlobal("navigator", { userAgent: "", platform: "" });
    render(<DownloadButton variant="primary" />);
    const link = screen.getByRole("link") as HTMLAnchorElement;
    await waitFor(() => {
      expect(link.href).toBe("https://storymoss.top/releases/");
    });
  });
});

describe("download helpers", () => {
  it("detects windows", () => {
    vi.stubGlobal("navigator", {
      userAgent: "Windows NT 10.0",
      platform: "Win32",
    });
    expect(detectPlatform()).toBe("windows");
  });

  it("detects linux", () => {
    vi.stubGlobal("navigator", {
      userAgent: "X11; Linux x86_64",
      platform: "Linux x86_64",
    });
    expect(detectPlatform()).toBe("linux");
  });

  it("detects mac", () => {
    vi.stubGlobal("navigator", {
      userAgent: "Macintosh",
      platform: "MacIntel",
    });
    expect(detectPlatform()).toBe("mac");
  });

  it("returns fallback labels", () => {
    expect(downloadLabel("unknown", "立即下载")).toBe("立即下载");
    expect(downloadLabel("windows")).toBe("下载 Windows 版");
  });

  it("returns fallback url for unknown platform", () => {
    expect(downloadUrl("unknown", FALLBACK_RELEASE.urls)).toBe(
      "https://storymoss.top/releases/",
    );
  });
});
