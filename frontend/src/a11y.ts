const FOCUSABLE_SELECTOR = [
    "a[href]",
    "button:not([disabled])",
    "textarea:not([disabled])",
    "input:not([disabled])",
    "select:not([disabled])",
    "[tabindex]:not([tabindex='-1'])",
    "[contenteditable='true']",
].join(", ");

type FocusTrapTeardown = () => void;

/**
 * Constrains keyboard focus within the provided element until the returned teardown
 * function is invoked. Used for modals and overlays.
 */
export function trapFocus(container: HTMLElement): FocusTrapTeardown {
    const handleKeydown = (event: KeyboardEvent): void => {
        if (event.key !== "Tab") {
            return;
        }

        const focusable = getFocusableElements(container);
        if (focusable.length === 0) {
            event.preventDefault();
            return;
        }

        const first = focusable[0];
        const last = focusable[focusable.length - 1];
        const activeElement = document.activeElement as HTMLElement | null;

        if (event.shiftKey) {
            if (activeElement === first || !container.contains(activeElement)) {
                event.preventDefault();
                last.focus();
            }
            return;
        }

        if (activeElement === last) {
            event.preventDefault();
            first.focus();
        }
    };

    document.addEventListener("keydown", handleKeydown, true);

    return () => {
        document.removeEventListener("keydown", handleKeydown, true);
    };
}

function getFocusableElements(container: HTMLElement): HTMLElement[] {
    const elements = Array.from(container.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR));
    return elements.filter(
        (element) =>
            !element.hasAttribute("disabled") &&
            element.tabIndex !== -1 &&
            isElementVisible(element),
    );
}

function isElementVisible(element: HTMLElement): boolean {
    if (element.offsetParent !== null) {
        return true;
    }

    const rects = element.getClientRects();
    return rects.length > 0;
}

let prefersReducedMotionFlag = false;
let reduceMotionQuery: MediaQueryList | null = null;
let reduceMotionInitialized = false;

/**
 * Returns true when the user requested reduced motion in their OS/browser preferences.
 */
export function prefersReducedMotion(): boolean {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
        return false;
    }

    if (!reduceMotionInitialized) {
        reduceMotionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
        prefersReducedMotionFlag = reduceMotionQuery.matches;

        const updatePreference = (event: MediaQueryList | MediaQueryListEvent): void => {
            prefersReducedMotionFlag = event.matches;
        };

        if (typeof reduceMotionQuery.addEventListener === "function") {
            reduceMotionQuery.addEventListener("change", updatePreference);
        } else if (typeof reduceMotionQuery.addListener === "function") {
            reduceMotionQuery.addListener(updatePreference);
        }

        reduceMotionInitialized = true;
    }

    return prefersReducedMotionFlag;
}
