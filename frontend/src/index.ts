/**
 * Entry point for the Iron Shield dashboard frontend bundle.
 * Sets up global styles, favicon handling, and bootstraps the clock and uptime modules.
 */
import "./styles/base.css";
import "./styles/clock.css";
import "./styles/filters.css";
import "./styles/notifications.css";
import "./styles/sites.css";
import "./styles/settings.css";
import faviconUrl from "./favicon.svg";
import { initClock } from "./clock.ts";
import { initUptimeSSE } from "./uptime.ts";
import { initSiteFilters } from "./filters.ts";
import { initSiteStatusNotifications } from "./notifications.ts";
import { initSettingsPanel } from "./settings.ts";
import { initSiteClickTelemetry } from "./telemetry.ts";

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
  initSiteFilters();
  initSiteStatusNotifications();
  initSettingsPanel();
  initUptimeSSE();
  initSiteClickTelemetry();
}

document.addEventListener("DOMContentLoaded", bootstrap);
