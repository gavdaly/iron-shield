interface UptimeInfo {
  site_id: string;
  status: string;
  uptime_percentage: number;
}

export function initUptimeSSE(): void {
  const eventSource = new EventSource("/uptime");

  eventSource.onmessage = (event) => {
    try {
      const payload = JSON.parse(event.data) as unknown;
      if (!Array.isArray(payload)) {
        return;
      }

      payload.forEach((entry) => {
        if (isUptimeInfo(entry)) {
          updateSiteCard(entry);
        }
      });
    } catch (error) {
      console.error("Failed to parse uptime data", error);
    }
  };

  eventSource.onerror = (error) => {
    console.error("Uptime SSE connection error", error);
  };
}

function updateSiteCard(info: UptimeInfo): void {
  const siteCards = document.querySelectorAll<HTMLElement>(".site-card");

  siteCards.forEach((card) => {
    const siteName = card.querySelector("a span")?.textContent?.trim();
    if (!siteName || siteName !== info.site_id) {
      return;
    }

    const uptimeElement = card.querySelector<HTMLElement>(".uptime");
    if (!uptimeElement) {
      return;
    }

    uptimeElement.className = `uptime ${info.status}`;
    const statusText = `${info.status.charAt(0).toUpperCase()}${info.status.slice(1)}`;
    const percentage = info.uptime_percentage.toFixed(1);
    uptimeElement.innerHTML = `
      <span class="status-text">${statusText}</span>
      <span class="uptime-percentage">${percentage}%</span>
    `;
  });
}

function isUptimeInfo(value: unknown): value is UptimeInfo {
  if (typeof value !== "object" || value === null) {
    return false;
  }

  const candidate = value as Partial<UptimeInfo>;
  return (
    typeof candidate.site_id === "string" &&
    typeof candidate.status === "string" &&
    typeof candidate.uptime_percentage === "number"
  );
}
