/**
 * Data payload describing the current uptime status of a site card.
 */
interface HistorySample {
  status: string;
  response_time_ms?: number | null;
}

interface UptimeInfo {
  site_id: string;
  status: string;
  uptime_percentage: number;
  history: HistorySample[];
  response_time_ms?: number | null;
}

const MAX_HISTORY_BARS = 12;

/**
 * Establish the SSE connection for uptime updates and update cards on new events.
 */
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

/**
 * Apply an uptime update to the matching site card if it exists.
 */
function updateSiteCard(info: UptimeInfo): void {
  const siteCards = document.querySelectorAll<HTMLElement>(".site-card");

  siteCards.forEach((card) => {
    const siteName = card.querySelector<HTMLElement>(".site-name")?.textContent?.trim();
    if (!siteName || siteName !== info.site_id) {
      return;
    }

    const uptimeElement = card.querySelector<HTMLElement>(".uptime");
    const historyElement = card.querySelector<HTMLElement>(".uptime-history");
    if (!uptimeElement) {
      return;
    }

    const normalizedStatus = info.status.toLowerCase();

    uptimeElement.className = `uptime ${normalizedStatus}`;
    const statusText = formatStatus(normalizedStatus);
    const percentage = info.uptime_percentage.toFixed(1);
    uptimeElement.innerHTML = `
      <span class="status-text">${statusText}</span>
      <span class="uptime-percentage">${percentage}%</span>
    `;

    if (historyElement) {
      renderHistory(historyElement, info.history, info.site_id);
    }
  });
}

function renderHistory(element: HTMLElement, history: HistorySample[], siteId: string): void {
  const recentHistory = history.slice(-MAX_HISTORY_BARS);
  element.innerHTML = "";

  if (recentHistory.length === 0) {
    const placeholder = document.createElement("span");
    placeholder.className = "uptime-history-placeholder";
    placeholder.textContent = "No history yet";
    element.appendChild(placeholder);
    element.setAttribute("aria-label", "No uptime history yet");
    return;
  }

  const fragment = document.createDocumentFragment();
  const siteSlug = slugifyForId(siteId);
  recentHistory.forEach((sample, index) => {
    const normalizedStatus = sample.status.toLowerCase();
    const wrapper = document.createElement("span");
    wrapper.className = "history-bar-wrapper";

    const bar = document.createElement("span");
    bar.className = `history-bar ${normalizedStatus}`;
    bar.setAttribute("tabindex", "0");
    const anchorId = `${siteSlug}-history-${index}`;
    bar.id = anchorId;
    bar.setAttribute(
      "aria-label",
      `${formatStatus(normalizedStatus)} – ${formatPopoverDetail(normalizedStatus, sample.response_time_ms)}`,
    );

    const popover = document.createElement("dialog");
    popover.className = "history-bar-popover";
    popover.setAttribute("popover", "manual");
    popover.innerHTML = `
      <div class="popover-status">${formatStatus(normalizedStatus)}</div>
      <div class="popover-label">${formatPopoverDetail(normalizedStatus, sample.response_time_ms)}</div>
    `;

    attachPopoverTriggers(bar, popover as PopoverElement);

    wrapper.append(bar, popover);
    fragment.appendChild(wrapper);
  });

  element.appendChild(fragment);
  element.setAttribute("aria-label", `Last ${recentHistory.length} checks for ${siteId}`);
}

function formatStatus(status: string): string {
  if (!status) {
    return "Unknown";
  }

  return `${status.charAt(0).toUpperCase()}${status.slice(1)}`;
}

function formatPopoverDetail(status: string, responseTime?: number | null): string {
  if (status === "down") {
    return "Unreachable";
  }

  if (status === "loading") {
    return "Checking...";
  }

  return `Response ${formatResponseTime(responseTime)}`;
}

function formatResponseTime(value?: number | null): string {
  if (typeof value !== "number" || Number.isNaN(value) || value < 0) {
    return "Not available";
  }

  if (value >= 1000) {
    const seconds = value / 1000;
    return `${seconds.toFixed(2)} s`;
  }

  return `${Math.round(value)} ms`;
}

type PopoverElement = HTMLDialogElement & {
  showPopover?: () => void;
  hidePopover?: () => void;
};

function attachPopoverTriggers(anchor: HTMLElement, popover: PopoverElement): void {
  const show = typeof popover.showPopover === "function" ? popover.showPopover.bind(popover) : null;
  const hide = typeof popover.hidePopover === "function" ? popover.hidePopover.bind(popover) : null;

  if (!show || !hide) {
    return;
  }

  const showWithPosition = (): void => {
    positionPopover(popover, anchor);
    show();
  };

  const hidePopover = (): void => {
    hide();
  };

  anchor.addEventListener("mouseenter", showWithPosition);
  anchor.addEventListener("focus", showWithPosition);
  anchor.addEventListener("mouseleave", hidePopover);
  anchor.addEventListener("blur", hidePopover);
}

function positionPopover(popover: HTMLElement, anchor: HTMLElement): void {
  const rect = anchor.getBoundingClientRect();
  popover.style.position = "fixed";
  popover.style.left = `${rect.left + rect.width / 2}px`;
  popover.style.top = `${rect.top - 8}px`;
  popover.style.transform = "translate(-50%, -100%)";
}

function slugifyForId(raw: string): string {
  return raw.toLowerCase().replace(/\s+/g, "-").replace(/[^a-z0-9-_]/g, "");
}

/**
 * Type guard used to ensure parsed SSE payloads have the expected shape.
 */
function isUptimeInfo(value: unknown): value is UptimeInfo {
  if (typeof value !== "object" || value === null) {
    return false;
  }

  const candidate = value as Partial<UptimeInfo>;
  const isHistoryValid =
    Array.isArray(candidate.history) &&
    candidate.history.every((value) => isHistorySample(value));

  return (
    typeof candidate.site_id === "string" &&
    typeof candidate.status === "string" &&
    typeof candidate.uptime_percentage === "number" &&
    isHistoryValid &&
    (candidate.response_time_ms === undefined ||
      candidate.response_time_ms === null ||
      typeof candidate.response_time_ms === "number")
  );
}

function isHistorySample(value: unknown): value is HistorySample {
  if (typeof value !== "object" || value === null) {
    return false;
  }

  const sample = value as Partial<HistorySample>;
  return (
    typeof sample.status === "string" &&
    (sample.response_time_ms === undefined ||
      sample.response_time_ms === null ||
      typeof sample.response_time_ms === "number")
  );
}
