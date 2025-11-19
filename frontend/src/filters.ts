type SiteCard = HTMLElement & {
  dataset: DOMStringMap & {
    siteName?: string;
    tags?: string;
  };
};

interface TagMeta {
  label: string;
  value: string;
}

/**
 * Bootstraps the search input, tag chips, and keyboard shortcut.
 */
export function initSiteFilters(): void {
  if (typeof document === "undefined") {
    return;
  }

  const siteList = document.getElementById("sites");
  const searchInput = document.getElementById("site-search") as HTMLInputElement | null;
  const tagContainer = document.getElementById("tag-filter-chips");
  const tagField = document.getElementById("tag-filter-field");
  const emptyState = document.getElementById("site-filter-empty");
  setupFilterModal(searchInput);

  if (!siteList || !searchInput || !tagContainer || !emptyState) {
    return;
  }

  const siteCards = Array.from(siteList.querySelectorAll<SiteCard>(".site-card"));
  if (siteCards.length === 0) {
    tagField?.setAttribute("hidden", "true");
    emptyState.hidden = false;
    emptyState.textContent = "No sites configured yet.";
    return;
  }

  const allTags = collectTagMetadata(siteCards);
  const activeTags = new Set<string>();

  if (allTags.length === 0) {
    tagField?.setAttribute("hidden", "true");
  } else {
    renderTagButtons(allTags, tagContainer, activeTags, () => {
      applyFilters(siteCards, searchInput, activeTags, emptyState);
    });
  }

  searchInput.addEventListener("input", () => {
    applyFilters(siteCards, searchInput, activeTags, emptyState);
  });

  wireSearchShortcut(searchInput);
  applyFilters(siteCards, searchInput, activeTags, emptyState);
}

function setupFilterModal(searchInput: HTMLInputElement | null): void {
  const overlay = document.getElementById("site-filters-overlay");
  const openButton = document.getElementById("site-filters-toggle") as HTMLButtonElement | null;
  const closeButton = document.getElementById("site-filters-close") as HTMLButtonElement | null;
  const panel = overlay?.querySelector<HTMLElement>("#site-filters-panel") ?? null;

  if (!overlay || !panel || !openButton) {
    return;
  }

  let onDocumentKeydown: ((event: KeyboardEvent) => void) | null = null;

  const openModal = (): void => {
    if (!overlay.hidden) {
      return;
    }

    overlay.hidden = false;
    openButton.setAttribute("aria-expanded", "true");
    requestAnimationFrame(() => {
      searchInput?.focus();
      searchInput?.select();
    });

    onDocumentKeydown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        closeModal();
      }
    };

    document.addEventListener("keydown", onDocumentKeydown);
  };

  const closeModal = (): void => {
    if (overlay.hidden) {
      return;
    }

    overlay.hidden = true;
    openButton.setAttribute("aria-expanded", "false");
    openButton.focus();

    if (onDocumentKeydown) {
      document.removeEventListener("keydown", onDocumentKeydown);
      onDocumentKeydown = null;
    }
  };

  openButton.addEventListener("click", () => {
    if (overlay.hidden) {
      openModal();
    } else {
      closeModal();
    }
  });
  closeButton?.addEventListener("click", closeModal);

  overlay.addEventListener("click", (event) => {
    if (event.target === overlay) {
      closeModal();
    }
  });
}

function collectTagMetadata(cards: SiteCard[]): TagMeta[] {
  const map = new Map<string, string>();
  cards.forEach((card) => {
    const rawTags = card.dataset.tags ?? "";
    rawTags
      .split(",")
      .map((tag) => tag.trim())
      .filter((tag) => tag.length > 0)
      .forEach((tag) => {
        const normalized = tag.toLowerCase();
        if (!map.has(normalized)) {
          map.set(normalized, tag);
        }
      });
  });

  return Array.from(map.entries())
    .map(([value, label]) => ({ value, label }))
    .sort((a, b) => a.label.localeCompare(b.label));
}

function renderTagButtons(
  tags: TagMeta[],
  container: HTMLElement,
  state: Set<string>,
  onChange: () => void,
): void {
  container.innerHTML = "";
  tags.forEach((tag) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "tag-filter";
    button.textContent = tag.label;
    button.dataset.tagValue = tag.value;
    button.setAttribute("aria-pressed", "false");
    button.setAttribute("role", "option");

    button.addEventListener("click", () => {
      toggleTag(button, tag.value, state);
      onChange();
    });

    button.addEventListener("keydown", (event) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        toggleTag(button, tag.value, state);
        onChange();
      }
    });

    container.appendChild(button);
  });
}

function toggleTag(button: HTMLElement, tag: string, state: Set<string>): void {
  const normalized = tag.toLowerCase();
  if (state.has(normalized)) {
    state.delete(normalized);
    button.setAttribute("aria-pressed", "false");
    button.classList.remove("tag-filter--active");
  } else {
    state.add(normalized);
    button.setAttribute("aria-pressed", "true");
    button.classList.add("tag-filter--active");
  }
}

function applyFilters(
  cards: SiteCard[],
  input: HTMLInputElement,
  activeTags: Set<string>,
  emptyState: HTMLElement,
): void {
  const query = input.value.trim().toLowerCase();
  const hasQuery = query.length > 0;
  const hasTagFilters = activeTags.size > 0;
  let visibleCount = 0;

  cards.forEach((card) => {
    const cardName = (card.dataset.siteName || card.textContent || "").toLowerCase();
    const cardTags = getCardTags(card);

    const matchesQuery =
      !hasQuery ||
      cardName.includes(query) ||
      cardTags.some((tag) => tag.includes(query));

    const matchesTags =
      !hasTagFilters || cardTags.some((tag) => activeTags.has(tag));

    const shouldShow = matchesQuery && matchesTags;
    card.hidden = !shouldShow;

    if (shouldShow) {
      visibleCount += 1;
    }
  });

  emptyState.hidden = visibleCount !== 0;
}

function getCardTags(card: SiteCard): string[] {
  const rawTags = card.dataset.tags ?? "";
  if (!rawTags) {
    return [];
  }

  return rawTags
    .split(",")
    .map((tag) => tag.trim().toLowerCase())
    .filter((tag) => tag.length > 0);
}

function wireSearchShortcut(input: HTMLInputElement): void {
  document.addEventListener("keydown", (event) => {
    if (event.key !== "/" || event.altKey || event.ctrlKey || event.metaKey || event.isComposing) {
      return;
    }

    const activeElement = document.activeElement;
    if (activeElement && isEditableElement(activeElement)) {
      return;
    }

    event.preventDefault();
    input.focus();
    input.select();
  });
}

function isEditableElement(element: Element): boolean {
  if (element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement) {
    return true;
  }

  const contentEditableValue = element.getAttribute("contenteditable");
  return contentEditableValue === "" || contentEditableValue === "true";
}
