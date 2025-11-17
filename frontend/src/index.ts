import "./style.css";
import faviconUrl from "./favicon.svg";
import { initClock } from "./clock.ts";
import { initUptimeSSE } from "./uptime.ts";

function applyFavicon(): void {
  const faviconLink = document.querySelector<HTMLLinkElement>("link[rel='icon']");
  if (faviconLink) {
    faviconLink.href = faviconUrl;
  }
}

function bootstrap(): void {
  applyFavicon();
  initClock();
  initUptimeSSE();
}

document.addEventListener("DOMContentLoaded", bootstrap);
