import type { ReactNode, AnchorHTMLAttributes } from "react";
import { useEffect, useState } from "react";
import { useLatestRelease, type ReleaseUrls } from "../hooks/useLatestRelease";

const RELEASES_INDEX = "https://storymoss.top/releases/";

export type Platform = "mac" | "macIntel" | "windows" | "linux" | "unknown";

export function detectPlatform(): Platform {
  if (typeof navigator === "undefined") return "unknown";

  const ua = navigator.userAgent.toLowerCase();
  const platform = navigator.platform?.toLowerCase() || "";

  if (ua.includes("win")) return "windows";
  if (ua.includes("linux")) return "linux";
  if (platform.startsWith("mac") || ua.includes("mac")) {
    // Modern Apple Silicon Macs report MacIntel in user agent due to Rosetta,
    // but navigator.platform is also MacIntel. Default to Apple Silicon for
    // current Mac users; Intel users can pick the x64 build from releases.
    return "mac";
  }

  return "unknown";
}

export function downloadUrl(platform: Platform, urls: ReleaseUrls): string {
  if (platform === "windows") return urls.windows;
  if (platform === "linux") return urls.linux;
  if (platform === "mac") return urls.mac;
  // macIntel and unknown fall back to the releases page so users can pick a build.
  return RELEASES_INDEX;
}

export function downloadLabel(
  platform: Platform,
  fallback = "免费下载",
): string {
  if (platform === "mac" || platform === "macIntel") return "下载 macOS 版";
  if (platform === "windows") return "下载 Windows 版";
  if (platform === "linux") return "下载 Linux 版";
  return fallback;
}

type DownloadButtonProps = AnchorHTMLAttributes<HTMLAnchorElement> & {
  variant: "primary" | "secondary";
  children?: ReactNode;
  fallbackLabel?: string;
};

export function DownloadButton({
  variant,
  children,
  fallbackLabel,
  className = "",
  ...rest
}: DownloadButtonProps) {
  const [platform, setPlatform] = useState<Platform>("unknown");
  const release = useLatestRelease();

  useEffect(() => {
    setPlatform(detectPlatform());
  }, []);

  const base =
    "inline-flex items-center justify-center rounded-full px-6 py-3 text-sm font-medium transition-[transform,background-color,color] duration-200 active:scale-95 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-moss focus-visible:ring-offset-2 focus-visible:ring-offset-canvas";
  const styles =
    variant === "primary"
      ? "bg-moss text-canvas [@media(hover:hover)]:hover:bg-moss-soft"
      : "surface-2 border border-subtle text-paper [@media(hover:hover)]:hover:surface-3";

  const url = downloadUrl(platform, release.urls);
  const label = children ?? downloadLabel(platform, fallbackLabel);

  return (
    <a
      href={url}
      target="_blank"
      rel="noreferrer"
      className={`${base} ${styles} ${className}`}
      {...rest}
    >
      {label}
    </a>
  );
}
