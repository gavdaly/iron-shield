/**
 * Entry point for the Iron Shield dashboard frontend bundle.
 * Sets up global styles, favicon handling, and bootstraps the clock and uptime modules.
 */
import "./style.css";
import faviconUrl from "./favicon.svg";
import { initClock } from "./clock.ts";
import { initUptimeSSE } from "./uptime.ts";

/**
 * Replace the static favicon reference with the bundled asset URL.
 */
function applyFavicon(): void {
  const faviconLink = document.querySelector<HTMLLinkElement>("link[rel='icon']");
  if (faviconLink) {
    faviconLink.href = faviconUrl;
  }
}

/**
 * Initialize the frontend widgets after the DOM is ready.
 */
function bootstrap(): void {
  applyFavicon();
  initClock();
  initUptimeSSE();
}

document.addEventListener("DOMContentLoaded", bootstrap);
