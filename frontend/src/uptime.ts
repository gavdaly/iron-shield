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
const HISTORY_ANIMATION_DURATION = 420;

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
    const isLoadingStatus = normalizedStatus === "loading";

    if (!isLoadingStatus) {
      uptimeElement.className = `uptime ${normalizedStatus}`;
      const statusText = formatStatus(normalizedStatus);
      const percentage = info.uptime_percentage.toFixed(1);
      uptimeElement.innerHTML = `
        <span class="status-text">${statusText}</span>
        <span class="uptime-percentage">${percentage}%</span>
      `;
    }

    if (historyElement) {
      renderHistory(historyElement, info.history, info.site_id);
    }
  });
}

function renderHistory(element: HTMLElement, history: HistorySample[], siteId: string): void {
  const recentHistory = history.slice(-MAX_HISTORY_BARS);
  const historyKey = createHistoryKey(recentHistory);

  if (recentHistory.length === 0) {
    element.innerHTML = "";
    const placeholder = document.createElement("span");
    placeholder.className = "uptime-history-placeholder";
    placeholder.textContent = "No history yet";
    element.appendChild(placeholder);
    element.setAttribute("aria-label", "No uptime history yet");
    delete element.dataset.historyKey;
    delete element.dataset.pendingHistory;
    delete element.dataset.animating;
    return;
  }

  if (element.dataset.animating === "true") {
    element.dataset.pendingHistory = JSON.stringify(recentHistory);
    return;
  }

  const previousKey = element.dataset.historyKey;
  if (previousKey === historyKey) {
    return;
  }
  const existingWrapperCount = element.querySelectorAll(".history-bar-wrapper").length;
  const shouldAnimateShift =
    Boolean(previousKey && previousKey !== historyKey) &&
    recentHistory.length === MAX_HISTORY_BARS &&
    existingWrapperCount === MAX_HISTORY_BARS;

  if (shouldAnimateShift) {
    animateHistoryTransition(element, recentHistory, siteId, historyKey);
    return;
  }

  const shouldHighlightNew = Boolean(previousKey && previousKey !== historyKey);
  buildHistoryBars(element, recentHistory, siteId, {
    highlightNew: shouldHighlightNew,
    historyKey,
  });
}

interface HistoryBuildOptions {
  highlightNew?: boolean;
  historyKey: string;
}

function buildHistoryBars(
  element: HTMLElement,
  history: HistorySample[],
  siteId: string,
  options: HistoryBuildOptions,
): void {
  element.innerHTML = "";
  const fragment = document.createDocumentFragment();
  const siteSlug = slugifyForId(siteId);

  history.forEach((sample, index) => {
    const normalizedStatus = sample.status.toLowerCase();
    const wrapper = document.createElement("span");
    wrapper.className = "history-bar-wrapper";
    if (options.highlightNew && index === history.length - 1) {
      wrapper.classList.add("history-bar-wrapper--incoming");
    }

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
  element.setAttribute("aria-label", `Last ${history.length} checks for ${siteId}`);
  element.dataset.historyKey = options.historyKey;
}

function animateHistoryTransition(
  element: HTMLElement,
  history: HistorySample[],
  siteId: string,
  historyKey: string,
): void {
  const wrappers = Array.from(element.querySelectorAll<HTMLElement>(".history-bar-wrapper"));
  if (wrappers.length === 0) {
    buildHistoryBars(element, history, siteId, { historyKey });
    return;
  }

  element.dataset.animating = "true";
  const [first, ...rest] = wrappers;
  first.classList.add("history-bar-wrapper--falling");
  rest.forEach((wrapper) => wrapper.classList.add("history-bar-wrapper--shifting"));

  window.setTimeout(() => {
    delete element.dataset.animating;
    buildHistoryBars(element, history, siteId, { highlightNew: true, historyKey });
    flushPendingHistory(element, siteId);
  }, HISTORY_ANIMATION_DURATION);
}

function flushPendingHistory(element: HTMLElement, siteId: string): void {
  const pending = element.dataset.pendingHistory;
  if (!pending) {
    return;
  }

  delete element.dataset.pendingHistory;
  try {
    const parsed = JSON.parse(pending) as unknown;
    if (Array.isArray(parsed) && parsed.every((value) => isHistorySample(value))) {
      renderHistory(element, parsed, siteId);
    }
  } catch {
    // Ignore malformed payloads.
  }
}

function createHistoryKey(history: HistorySample[]): string {
  if (history.length === 0) {
    return "empty";
  }

  return history
    .map((sample) => `${sample.status}:${sample.response_time_ms ?? "na"}`)
    .join("|");
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
