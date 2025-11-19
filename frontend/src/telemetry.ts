interface ClickTelemetryPayload {
    site_name: string;
    site_url: string;
}

function sendClickTelemetry(payload: ClickTelemetryPayload): void {
    const body = JSON.stringify(payload);
    const endpoint = "/api/telemetry/click";

    if (navigator.sendBeacon) {
        const blob = new Blob([body], { type: "application/json" });
        navigator.sendBeacon(endpoint, blob);
        return;
    }

    fetch(endpoint, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body,
        keepalive: true,
    }).catch(() => {
        // Non-critical telemetry failure; ignore
    });
}

export function initSiteClickTelemetry(): void {
    if (typeof document === "undefined") {
        return;
    }

    const sitesContainer = document.getElementById("sites");
    if (!sitesContainer) {
        return;
    }

    sitesContainer.addEventListener("click", (event) => {
        const target = event.target as HTMLElement | null;
        const link = target?.closest<HTMLAnchorElement>("a.site-name");
        if (!link) {
            return;
        }

        const card = link.closest<HTMLElement>(".site-card");
        const siteName = card?.dataset.siteName || link.textContent || link.href;
        sendClickTelemetry({
            site_name: siteName,
            site_url: link.href,
        });
    });
}
