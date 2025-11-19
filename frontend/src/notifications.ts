/**
 * Handles browser-level notifications when a site's uptime status changes.
 */
import notificationIconUrl from "./favicon.svg";

const NOTIFICATION_BUTTON_SELECTOR = "#site-notification-toggle";
const NOTIFICATION_LABEL_SELECTOR = ".notification-button-label";
const NOTIFIABLE_STATUSES = new Set(["up", "down"]);

interface StatusUpdatePayload {
    siteId: string;
    status: string;
    responseTimeMs?: number | null;
}

class SiteStatusNotificationController {
    private readonly latestStatuses = new Map<string, string>();
    private button: HTMLButtonElement | null = null;
    private buttonLabel: HTMLElement | null = null;
    private readonly supported: boolean = typeof window !== "undefined" && "Notification" in window;

    init(): void {
        this.button = document.querySelector<HTMLButtonElement>(NOTIFICATION_BUTTON_SELECTOR);
        this.buttonLabel = this.button?.querySelector<HTMLElement>(NOTIFICATION_LABEL_SELECTOR) ?? null;

        if (!this.button) {
            return;
        }

        if (!this.supported) {
            this.button.hidden = true;
            return;
        }

        this.button.addEventListener("click", () => {
            if (Notification.permission === "default") {
                void this.requestPermission();
            }
        });

        this.updateButtonState();
    }

    handleStatusUpdate(update: StatusUpdatePayload): void {
        if (!this.supported) {
            return;
        }

        const normalizedStatus = update.status.toLowerCase();
        if (normalizedStatus === "loading") {
            return;
        }

        const previousStatus = this.latestStatuses.get(update.siteId);
        this.latestStatuses.set(update.siteId, normalizedStatus);

        if (!previousStatus || previousStatus === normalizedStatus) {
            return;
        }

        if (
            Notification.permission !== "granted" ||
            !NOTIFIABLE_STATUSES.has(normalizedStatus) ||
            !NOTIFIABLE_STATUSES.has(previousStatus)
        ) {
            return;
        }

        this.sendNotification(update.siteId, normalizedStatus, update.responseTimeMs);
    }

    private async requestPermission(): Promise<void> {
        try {
            await Notification.requestPermission();
        } catch (error) {
            console.error("Failed to request notification permission", error);
        } finally {
            this.updateButtonState();
        }
    }

    private updateButtonState(): void {
        if (!this.button || !this.supported) {
            return;
        }

        const permission = Notification.permission;
        switch (permission) {
            case "granted":
                this.applyButtonState("Alerts enabled", true);
                this.button.classList.add("site-notification-button--granted");
                this.button.setAttribute(
                    "aria-label",
                    "System notifications are enabled for site status changes",
                );
                break;
            case "denied":
                this.applyButtonState("Notifications blocked", true);
                this.button.classList.remove("site-notification-button--granted");
                this.button.setAttribute(
                    "aria-label",
                    "Browser notifications are blocked. Update your browser settings to enable alerts.",
                );
                break;
            default:
                this.applyButtonState("Enable alerts", false);
                this.button.classList.remove("site-notification-button--granted");
                this.button.setAttribute(
                    "aria-label",
                    "Enable system notifications for site status changes",
                );
        }
    }

    private applyButtonState(label: string, disabled: boolean): void {
        if (!this.button) {
            return;
        }

        this.button.disabled = disabled;
        if (this.buttonLabel) {
            this.buttonLabel.textContent = label;
        }
    }

    private sendNotification(siteId: string, status: string, responseTimeMs?: number | null): void {
        const isDown = status === "down";
        const title = isDown ? `${siteId} is unreachable` : `${siteId} is back online`;
        const responseText =
            typeof responseTimeMs === "number" ? `${responseTimeMs} ms response time.` : "Response time unavailable.";
        const body = isDown ? "Iron Shield cannot reach this site." : `Site responded successfully. ${responseText}`;

        try {
            new Notification(title, {
                body,
                icon: notificationIconUrl,
                tag: `site-status-${siteId}`,
            });
        } catch (error) {
            console.error("Failed to display site status notification", error);
        }
    }
}

const notificationController = new SiteStatusNotificationController();

export function initSiteStatusNotifications(): void {
    notificationController.init();
}

export function notifySiteStatusChange(update: StatusUpdatePayload): void {
    notificationController.handleStatusUpdate(update);
}
