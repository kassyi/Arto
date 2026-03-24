/**
 * Footnote Handler
 * Provides smooth scrolling to footnotes and a hover popover (tooltip) for quick reading.
 *
 * NOTE: Hash-only anchors (`<a href="#fn-1">`) are rewritten to
 * `<span data-hash-link="fn-1" class="footnote-reference-link">` by the Rust
 * post-processor (post_process.rs) to prevent WebView2 from treating them as
 * navigation events and opening an external browser.
 * This handler finds those spans and attaches scroll/popover behavior.
 */

const POPOVER_ID = "arto-footnote-popover";
const HIGHLIGHT_CLASS = "footnote-highlight";
const HIGHLIGHT_TIMEOUT_MS = 2000;

/**
 * Find the footnote definition element by ID.
 * Looks up at click/hover time so it works even if the DOM was mutated after setup.
 * pulldown-cmark uses id="fn-1" on the <li> element inside the footnote list.
 */
function findTargetDef(targetId: string): HTMLElement | null {
  // Use querySelectorAll to find ALL elements with the exact ID,
  // bypassing the first match if it happens to be an auto-generated header slug.
  const exactMatches = document.querySelectorAll<HTMLElement>(`[id="${CSS.escape(targetId)}"]`);
  for (const el of Array.from(exactMatches)) {
    if (!/^(H1|H2|H3|H4|H5|H6)$/i.test(el.tagName)) {
      return el;
    }
  }

  // Fallback: pulldown-cmark sometimes prepends "fn-"
  if (!targetId.startsWith("fn-")) {
    const fnMatches = document.querySelectorAll<HTMLElement>(`[id="fn-${CSS.escape(targetId)}"]`);
    for (const el of Array.from(fnMatches)) {
      if (!/^(H1|H2|H3|H4|H5|H6)$/i.test(el.tagName)) {
        return el;
      }
    }
  }

  return null;
}

export function setupFootnotes(container: HTMLElement): void {
  // Ensure the shared popover element exists in the document.
  // It is appended to <body> and positioned via fixed coordinates.
  let popover = document.getElementById(POPOVER_ID) as HTMLElement | null;
  if (!popover) {
    popover = document.createElement("div");
    popover.id = POPOVER_ID;
    popover.className = "footnote-popover";
    document.body.appendChild(popover);
  }
  const popoverEl = popover;

  // Find all hash-link spans rewritten by Rust's post-processor.
  // Match ALL data-hash-link spans — JS will skip back-references (fnref*).
  const references = container.querySelectorAll<HTMLElement>("span[data-hash-link]");
  console.debug(`[footnotes] setupFootnotes: found ${references.length} hash-link spans`);

  references.forEach((el) => {
    // Prevent attaching multiple listeners on re-render
    if (el.dataset.footnoteListenersAttached === "true") return;
    el.dataset.footnoteListenersAttached = "true";

    const targetId = el.dataset.hashLink;
    if (!targetId) return;

    // Skip back-reference links (fnref*) — they go back from definition to reference
    if (targetId.startsWith("fnref")) return;

    console.debug(`[footnotes] Attaching handlers for hash-link: ${targetId}`);

    // ── 1. Click: smooth scroll + highlight ─────────────────────────────────
    el.addEventListener("click", (e: MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();

      console.log("[footnotes] Clicked!", targetId);

      const targetDef = findTargetDef(targetId);
      if (!targetDef) return;

      // Scroll the *content area* scroller, not the window.
      // Explicitly calculating position against the .content container prevents
      // some WebView/browser edge cases where scrollIntoView scrolls the entire frame to top.
      const scrollContainer = document.querySelector(".content");
      if (scrollContainer) {
        const containerRect = scrollContainer.getBoundingClientRect();
        const targetRect = targetDef.getBoundingClientRect();
        // Calculate the element's top position within the scrollable content area
        const targetTop = targetRect.top - containerRect.top + scrollContainer.scrollTop;

        // Apply a small offset (e.g., 60px) so the footnote isn't cramped against the top bar
        scrollContainer.scrollTo({
          top: Math.max(0, targetTop - 60),
          behavior: "smooth"
        });
      } else {
        // Fallback
        targetDef.scrollIntoView({ behavior: "smooth", block: "nearest" });
      }

      // Flash highlight
      targetDef.classList.add(HIGHLIGHT_CLASS);
      setTimeout(() => targetDef.classList.remove(HIGHLIGHT_CLASS), HIGHLIGHT_TIMEOUT_MS);
    });

    // ── 2. Hover: popover tooltip ────────────────────────────────────────────
    let hideTimeout: ReturnType<typeof setTimeout>;

    el.addEventListener("mouseenter", () => {
      clearTimeout(hideTimeout);

      const targetDef = findTargetDef(targetId);
      if (!targetDef) return;

      // Clone and clean the definition content
      const clone = targetDef.cloneNode(true) as HTMLElement;

      // Remove back-reference links (↩) and any internal hash spans
      clone.querySelectorAll("span[data-hash-link], a[href^='#']").forEach((n) => n.remove());
      // Remove any explicit label elements
      clone.querySelectorAll(".footnote-backref, .footnote-label").forEach((n) => n.remove());

      popoverEl.innerHTML = clone.innerHTML;

      // Use fixed positioning (viewport-relative) — no scroll offset math needed
      popoverEl.style.position = "fixed";
      popoverEl.style.visibility = "hidden";
      popoverEl.classList.add("visible");

      const rect = el.getBoundingClientRect();
      const pw = popoverEl.offsetWidth;
      const ph = popoverEl.offsetHeight;
      const margin = 16;
      const vw = window.innerWidth;
      const vh = window.innerHeight;

      let top = rect.bottom + 8;
      let left = rect.left + rect.width / 2 - pw / 2;

      // Clamp horizontally
      left = Math.max(margin, Math.min(left, vw - pw - margin));

      // Flip above if not enough room below
      if (top + ph > vh - margin) {
        top = rect.top - ph - 8;
      }
      // Clamp vertically
      top = Math.max(margin, top);

      popoverEl.style.top = `${top}px`;
      popoverEl.style.left = `${left}px`;
      popoverEl.style.visibility = "";
    });

    el.addEventListener("mouseleave", () => {
      hideTimeout = setTimeout(() => {
        popoverEl.classList.remove("visible");
      }, 120);
    });
  });
}
