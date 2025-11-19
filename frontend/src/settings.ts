interface SiteConfig {
    name: string;
    url: string;
    category: string;
    tags: string[];
}

interface ConfigData {
    site_name: string;
    clock: string;
    opentelemetry_endpoint: string | null;
    sites: SiteConfig[];
}

type NotificationVariant = "success" | "error";

const BORDER_DEFAULT = "var(--color-border)";
const BORDER_ERROR = "var(--color-error)";

let configData: ConfigData | null = null;
let modalElement: HTMLElement | null = null;
let sitesListElement: HTMLElement | null = null;
let notificationElement: HTMLElement | null = null;
let lastFocusedTrigger: HTMLElement | null = null;

export function initSettingsPanel(): void {
    modalElement = document.getElementById("settings-modal");
    const openButton = document.getElementById("settings-open-button") as HTMLButtonElement | null;
    const parsedConfig = parseInitialConfig();

    if (!modalElement || !openButton || !parsedConfig) {
        return;
    }

    configData = parsedConfig;
    if (configData.opentelemetry_endpoint === undefined) {
        configData.opentelemetry_endpoint = null;
    }
    sitesListElement = modalElement.querySelector<HTMLElement>("#settings-sites-list");
    notificationElement = modalElement.querySelector<HTMLElement>("#settings-notification-area");

    openButton.addEventListener("click", () => openModal(openButton));
    modalElement
        .querySelectorAll<HTMLElement>("[data-settings-close]")
        .forEach((element) => element.addEventListener("click", closeModal));

    modalElement.addEventListener("keydown", (event: KeyboardEvent) => {
        if (event.key === "Escape") {
            event.preventDefault();
            closeModal();
        }
    });

    applyInitialValues();
    renderSites();
    bindFormHandlers();
}

function parseInitialConfig(): ConfigData | null {
    const configScript = document.getElementById("initial-config");
    if (!configScript?.textContent) {
        return null;
    }

    try {
        return JSON.parse(configScript.textContent) as ConfigData;
    } catch (error) {
        console.error("Failed to parse initial config payload", error);
        return null;
    }
}

function openModal(trigger?: HTMLElement | null): void {
    if (!modalElement) {
        return;
    }

    lastFocusedTrigger = trigger ?? null;
    modalElement.hidden = false;
    modalElement.setAttribute("aria-hidden", "false");
    document.body.classList.add("settings-modal-open");

    const siteNameInput = modalElement.querySelector<HTMLInputElement>("#settings-site-name");
    siteNameInput?.focus();
}

function closeModal(): void {
    if (!modalElement) {
        return;
    }

    modalElement.hidden = true;
    modalElement.setAttribute("aria-hidden", "true");
    document.body.classList.remove("settings-modal-open");
    hideNotification();
    lastFocusedTrigger?.focus();
}

function applyInitialValues(): void {
    if (!configData) {
        return;
    }

    const siteNameInput = document.getElementById("settings-site-name") as HTMLInputElement | null;
    const clockSelect = document.getElementById("settings-clock-format") as HTMLSelectElement | null;
    const telemetryInput = document.getElementById(
        "settings-telemetry-endpoint",
    ) as HTMLInputElement | null;

    if (siteNameInput) {
        siteNameInput.value = configData.site_name;
        siteNameInput.addEventListener("input", () => {
            configData!.site_name = siteNameInput.value;
            clearInputError(siteNameInput);
        });
    }

    if (clockSelect) {
        clockSelect.value = configData.clock;
        clockSelect.addEventListener("change", () => {
            configData!.clock = clockSelect.value;
        });
    }

    if (telemetryInput) {
        telemetryInput.value = configData.opentelemetry_endpoint ?? "";
        telemetryInput.addEventListener("input", () => {
            const trimmed = telemetryInput.value.trim();
            configData!.opentelemetry_endpoint = trimmed.length > 0 ? trimmed : null;
            validateOptionalUrlInput(telemetryInput);
        });
        telemetryInput.addEventListener("blur", () => validateOptionalUrlInput(telemetryInput));
    }
}

function bindFormHandlers(): void {
    const addSiteButton = document.getElementById("add-site-btn") as HTMLButtonElement | null;
    const newSiteNameInput = document.getElementById("new-site-name") as HTMLInputElement | null;
    const newSiteUrlInput = document.getElementById("new-site-url") as HTMLInputElement | null;
    const saveButton = document.getElementById("settings-save-button") as HTMLButtonElement | null;
    const resetButton = document.getElementById("settings-reset-button") as HTMLButtonElement | null;

    newSiteNameInput?.addEventListener("input", () => validateRequiredInput(newSiteNameInput));
    newSiteUrlInput?.addEventListener("input", () => validateUrlInput(newSiteUrlInput));

    addSiteButton?.addEventListener("click", handleAddSite);
    saveButton?.addEventListener("click", saveSettings);
    resetButton?.addEventListener("click", resetSettings);
}

function handleAddSite(): void {
    if (!configData) {
        return;
    }

    const nameInput = document.getElementById("new-site-name") as HTMLInputElement | null;
    const urlInput = document.getElementById("new-site-url") as HTMLInputElement | null;
    const categoryInput = document.getElementById("new-site-category") as HTMLInputElement | null;
    const tagsInput = document.getElementById("new-site-tags") as HTMLInputElement | null;

    if (!nameInput || !urlInput || !categoryInput || !tagsInput) {
        return;
    }

    const isNameValid = validateRequiredInput(nameInput);
    const isUrlValid = validateUrlInput(urlInput);

    if (!isNameValid || !isUrlValid) {
        showNotification("Please fix the highlighted fields before adding a site.", "error");
        return;
    }

    const newSite: SiteConfig = {
        name: nameInput.value.trim(),
        url: urlInput.value.trim(),
        category: categoryInput.value.trim(),
        tags: parseTags(tagsInput.value),
    };

    configData.sites.push(newSite);
    renderSites();

    nameInput.value = "";
    urlInput.value = "";
    categoryInput.value = "";
    tagsInput.value = "";
    clearInputError(nameInput);
    clearInputError(urlInput);
}

function parseTags(rawTags: string): string[] {
    return rawTags
        .split(",")
        .map((tag) => tag.trim())
        .filter((tag) => tag.length > 0);
}

function renderSites(): void {
    if (!configData || !sitesListElement) {
        return;
    }

    sitesListElement.innerHTML = "";

    configData.sites.forEach((site, index) => {
        sitesListElement?.appendChild(createSiteItem(site, index));
    });
}

function createSiteItem(site: SiteConfig, index: number): HTMLElement {
    const wrapper = document.createElement("div");
    wrapper.className = "site-item";
    wrapper.dataset.siteIndex = `${index}`;

    const nameInput = createLabeledInput(
        `site-name-${index}`,
        "Site Name",
        site.name,
        (value) => {
            site.name = value;
        },
        validateRequiredInput,
    );

    const urlInput = createLabeledInput(
        `site-url-${index}`,
        "URL",
        site.url,
        (value) => {
            site.url = value;
        },
        validateUrlInput,
        "text",
        "https://example.com",
    );

    const categoryInput = createLabeledInput(
        `site-category-${index}`,
        "Category",
        site.category,
        (value) => {
            site.category = value;
        },
        undefined,
        "text",
        "Category (e.g., Work)",
    );

    const tagsGroup = document.createElement("div");
    tagsGroup.className = "form-group";
    const tagsLabel = document.createElement("label");
    tagsLabel.textContent = "Tags";
    tagsLabel.htmlFor = `site-tags-${index}`;
    const tagsContainer = document.createElement("div");
    tagsContainer.className = "existing-tags";
    tagsContainer.id = `site-tags-${index}`;

    site.tags.forEach((tag) => {
        tagsContainer.appendChild(createTagChip(tag, index));
    });

    const tagInputRow = document.createElement("div");
    tagInputRow.className = "tag-input-form";
    const tagInput = document.createElement("input");
    tagInput.type = "text";
    tagInput.placeholder = "Add tag";
    tagInput.autocomplete = "off";
    tagInput.addEventListener("keypress", (event: KeyboardEvent) => {
        if (event.key === "Enter") {
            event.preventDefault();
            handleAddTag(index, tagInput, tagsContainer);
        }
    });

    const addTagButton = document.createElement("button");
    addTagButton.type = "button";
    addTagButton.className = "btn add-tag-btn";
    addTagButton.textContent = "Add";
    addTagButton.addEventListener("click", () => handleAddTag(index, tagInput, tagsContainer));

    tagInputRow.append(tagInput, addTagButton);
    tagsGroup.append(tagsLabel, tagsContainer, tagInputRow);

    const siteActions = document.createElement("div");
    siteActions.className = "site-actions";
    const deleteButton = document.createElement("button");
    deleteButton.type = "button";
    deleteButton.className = "btn btn-danger";
    deleteButton.textContent = "Delete";
    deleteButton.addEventListener("click", () => deleteSite(index));
    siteActions.appendChild(deleteButton);

    wrapper.append(nameInput, urlInput, categoryInput, tagsGroup, siteActions);
    return wrapper;
}

function createLabeledInput(
    id: string,
    labelText: string,
    value: string,
    onInput: (value: string) => void,
    validator?: (input: HTMLInputElement) => boolean,
    type = "text",
    placeholder?: string,
): HTMLElement {
    const group = document.createElement("div");
    group.className = "form-group";

    const label = document.createElement("label");
    label.htmlFor = id;
    label.textContent = labelText;

    const input = document.createElement("input");
    input.id = id;
    input.type = type;
    input.value = value;
    input.autocomplete = "off";
    if (placeholder) {
        input.placeholder = placeholder;
    }

    input.addEventListener("input", () => {
        onInput(input.value);
        if (validator) {
            validator(input);
        } else {
            clearInputError(input);
        }
    });

    if (validator) {
        input.addEventListener("blur", () => validator(input));
    }

    group.append(label, input);
    return group;
}

function handleAddTag(siteIndex: number, input: HTMLInputElement, container: HTMLElement): void {
    const value = input.value.trim();
    if (!configData || value.length === 0) {
        return;
    }

    configData.sites[siteIndex].tags.push(value);
    container.appendChild(createTagChip(value, siteIndex));
    input.value = "";
}

function createTagChip(tag: string, siteIndex: number): HTMLElement {
    const chip = document.createElement("span");
    chip.className = "tag-chip";
    chip.dataset.tagValue = tag;

    const text = document.createElement("span");
    text.textContent = tag;

    const removeButton = document.createElement("button");
    removeButton.type = "button";
    removeButton.className = "remove-btn";
    removeButton.textContent = "×";
    removeButton.setAttribute("aria-label", `Remove tag ${tag}`);
    removeButton.addEventListener("click", () => removeTag(siteIndex, tag, chip));

    chip.append(text, removeButton);
    return chip;
}

function removeTag(siteIndex: number, tagValue: string, chipElement: HTMLElement): void {
    if (!configData) {
        return;
    }

    configData.sites[siteIndex].tags = configData.sites[siteIndex].tags.filter(
        (tag) => tag !== tagValue,
    );
    chipElement.remove();
}

function deleteSite(siteIndex: number): void {
    if (!configData) {
        return;
    }

    const confirmed = window.confirm("Are you sure you want to delete this site?");
    if (!confirmed) {
        return;
    }

    configData.sites.splice(siteIndex, 1);
    renderSites();
}

function saveSettings(): void {
    if (!configData) {
        return;
    }

    hideNotification();

    const siteNameInput = document.getElementById("settings-site-name") as HTMLInputElement | null;
    const clockSelect = document.getElementById("settings-clock-format") as HTMLSelectElement | null;
    const telemetryInput = document.getElementById(
        "settings-telemetry-endpoint",
    ) as HTMLInputElement | null;

    if (siteNameInput) {
        configData.site_name = siteNameInput.value.trim();
        if (!validateRequiredInput(siteNameInput)) {
            showNotification("Site name cannot be empty.", "error");
            return;
        }
    }

    if (clockSelect) {
        configData.clock = clockSelect.value;
    }

    if (telemetryInput) {
        const trimmed = telemetryInput.value.trim();
        configData.opentelemetry_endpoint = trimmed.length > 0 ? trimmed : null;
    }

    const validationErrors = validateSites();
    if (telemetryInput && !validateOptionalUrlInput(telemetryInput)) {
        validationErrors.push("OpenTelemetry endpoint must be a valid URL.");
    }
    if (validationErrors.length > 0) {
        showNotification(`Validation errors:\n${validationErrors.join("\n")}`, "error");
        return;
    }

    const saveButton = document.getElementById("settings-save-button") as HTMLButtonElement | null;
    if (saveButton) {
        saveButton.disabled = true;
        saveButton.textContent = "Saving…";
    }

    fetch("/api/config", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(configData),
    })
        .then((response) => {
            if (!response.ok) {
                return response.text().then((text) => {
                    throw new Error(text || "Failed to save settings");
                });
            }

            showNotification("Settings saved successfully!", "success");
            setTimeout(() => {
                window.location.reload();
            }, 1500);
            return null;
        })
        .catch((error: Error) => {
            console.error("Error saving settings", error);
            showNotification(`Error saving settings: ${error.message}`, "error");
        })
        .finally(() => {
            if (saveButton) {
                saveButton.disabled = false;
                saveButton.textContent = "Save Settings";
            }
        });
}

function validateSites(): string[] {
    if (!configData) {
        return [];
    }

    const errors: string[] = [];

    configData.sites.forEach((site, index) => {
        const nameInput = document.getElementById(`site-name-${index}`) as HTMLInputElement | null;
        const urlInput = document.getElementById(`site-url-${index}`) as HTMLInputElement | null;

        if (!site.name.trim()) {
            errors.push(`Site ${index + 1}: Name is required.`);
            if (nameInput) {
                setInputError(nameInput, "Site name cannot be empty");
            }
        }

        if (!site.url.trim()) {
            errors.push(`Site ${index + 1}: URL is required.`);
            if (urlInput) {
                setInputError(urlInput, "URL cannot be empty");
            }
        } else if (!isValidUrl(site.url)) {
            errors.push(`Site ${index + 1}: Invalid URL format (${site.url}).`);
            if (urlInput) {
                setInputError(urlInput, "Please enter a valid URL (https://example.com)");
            }
        }
    });

    return errors;
}

function resetSettings(): void {
    const confirmed = window.confirm("Reset all changes? Any unsaved edits will be lost.");
    if (confirmed) {
        window.location.reload();
    }
}

function validateRequiredInput(input: HTMLInputElement): boolean {
    if (input.value.trim().length === 0) {
        setInputError(input, `${input.placeholder || "This field"} cannot be empty`);
        return false;
    }

    clearInputError(input);
    return true;
}

function validateUrlInput(input: HTMLInputElement): boolean {
    const value = input.value.trim();
    if (value.length === 0) {
        setInputError(input, "URL cannot be empty");
        return false;
    }

    if (!isValidUrl(value)) {
        setInputError(input, "Please enter a valid URL (https://example.com)");
        return false;
    }

    clearInputError(input);
    return true;
}

function validateOptionalUrlInput(input: HTMLInputElement): boolean {
    const value = input.value.trim();
    if (value.length === 0) {
        clearInputError(input);
        return true;
    }

    if (!isValidUrl(value)) {
        setInputError(input, "Please enter a valid URL (https://example.com)");
        return false;
    }

    clearInputError(input);
    return true;
}

function isValidUrl(candidate: string): boolean {
    try {
        const url = new URL(candidate);
        return url.protocol === "http:" || url.protocol === "https:";
    } catch {
        return false;
    }
}

function setInputError(input: HTMLInputElement, message: string): void {
    input.style.borderColor = BORDER_ERROR;
    input.title = message;
}

function clearInputError(input: HTMLInputElement): void {
    input.style.borderColor = BORDER_DEFAULT;
    input.title = "";
}

function showNotification(message: string, variant: NotificationVariant): void {
    if (!notificationElement) {
        return;
    }

    notificationElement.textContent = message;
    notificationElement.hidden = false;
    notificationElement.classList.toggle("settings-notification--success", variant === "success");
    notificationElement.classList.toggle("settings-notification--error", variant === "error");

    if (variant === "success") {
        window.setTimeout(() => hideNotification(), 5000);
    }
}

function hideNotification(): void {
    if (!notificationElement) {
        return;
    }

    notificationElement.hidden = true;
    notificationElement.textContent = "";
    notificationElement.classList.remove(
        "settings-notification--success",
        "settings-notification--error",
    );
}
