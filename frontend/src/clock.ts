/**
 * Utilities for rendering and updating the Iron Shield dashboard clock.
 */
export type ClockFormat = "12hour" | "24hour";

let clockInterval: number | undefined;

/**
 * Initialize the live clock and start ticking once per second.
 */
export function initClock(): void {
  const timeElement = document.getElementById("time");
  if (!(timeElement instanceof HTMLElement)) {
    return;
  }

  const format =
    (timeElement.dataset.format as ClockFormat | undefined) ?? "24hour";
  updateTimeAndBackground(timeElement, format);

  clockInterval = window.setInterval(() => {
    updateTimeAndBackground(timeElement, format);
  }, 1000);
}

/**
 * Update the clock text and time-of-day background based on the current time.
 */
function updateTimeAndBackground(
  element: HTMLElement,
  format: ClockFormat,
): void {
  const now = new Date();
  element.textContent = formatTime(now.getHours(), now.getMinutes(), format);
  updateTimeOfDayBackground(now);
}

/**
 * Format hours/minutes for either 12-hour or 24-hour display.
 */
function formatTime(
  hours: number,
  minutes: number,
  format: ClockFormat,
): string {
  const paddedMinutes = minutes.toString().padStart(2, "0");

  if (format === "12hour") {
    let displayHours = hours % 12;
    if (displayHours === 0) {
      displayHours = 12;
    }
    const ampm = hours >= 12 ? "PM" : "AM";
    return `${displayHours}:${paddedMinutes} ${ampm}`;
  }

  const paddedHours = hours.toString().padStart(2, "0");
  return `${paddedHours}:${paddedMinutes}`;
}

/**
 * Apply the data-time attribute used for background gradients.
 */
function updateTimeOfDayBackground(date: Date): void {
  const hour = date.getHours();

  if (hour >= 5 && hour < 12) {
    setDataTime("morning");
  } else if (hour >= 12 && hour < 17) {
    setDataTime("afternoon");
  } else if (hour >= 17 && hour < 21) {
    setDataTime("evening");
  } else {
    setDataTime("night");
  }
}

/**
 * Helper to mutate the body attribute safely.
 */
function setDataTime(value: string) {
  const body = document.body;
  body.setAttribute("data-time", value);
}

window.addEventListener("beforeunload", () => {
  if (clockInterval !== undefined) {
    window.clearInterval(clockInterval);
  }
});
