/// Scroll controller for Arto keybinding system.
///
/// Provides programmatic scroll control for the content area,
/// called from Rust via document::eval().

const SCROLL_LINE_HEIGHT = 60;
const SCROLL_HALF_PAGE_RATIO = 0.5;

function getContentElement(): HTMLElement | null {
  return document.querySelector(".content") as HTMLElement | null;
}

function scrollBy(el: HTMLElement, delta: number): void {
  el.scrollBy({ top: delta, behavior: "smooth" });
}

function scrollTo(el: HTMLElement, top: number): void {
  el.scrollTo({ top, behavior: "smooth" });
}

export function scrollDown(): void {
  const el = getContentElement();
  if (el) scrollBy(el, SCROLL_LINE_HEIGHT);
}

export function scrollUp(): void {
  const el = getContentElement();
  if (el) scrollBy(el, -SCROLL_LINE_HEIGHT);
}

export function scrollPageDown(): void {
  const el = getContentElement();
  if (el) scrollBy(el, el.clientHeight);
}

export function scrollPageUp(): void {
  const el = getContentElement();
  if (el) scrollBy(el, -el.clientHeight);
}

export function scrollHalfPageDown(): void {
  const el = getContentElement();
  if (el) scrollBy(el, el.clientHeight * SCROLL_HALF_PAGE_RATIO);
}

export function scrollHalfPageUp(): void {
  const el = getContentElement();
  if (el) scrollBy(el, -el.clientHeight * SCROLL_HALF_PAGE_RATIO);
}

export function scrollToTop(): void {
  const el = getContentElement();
  if (el) scrollTo(el, 0);
}

export function scrollToBottom(): void {
  const el = getContentElement();
  if (el) scrollTo(el, el.scrollHeight);
}

/**
 * Replaces native `element.scrollIntoView()` with a manual scroll position calculation.
 * This prevents the known Chromium overscroll bug where elements inside a container with `zoom`
 * cause `scrollTop` to exceed the `scrollHeight - clientHeight` bounds, resulting in blank space.
 *
 * @param el The target element to scroll to
 * @param block Where to align the element ('start', 'center', 'nearest')
 */
export function scrollIntoViewClamped(
  el: HTMLElement,
  block: "start" | "center" | "nearest" = "start",
): void {
  const container = getContentElement();
  if (!container || !el) return;

  const containerRect = container.getBoundingClientRect();
  const elRect = el.getBoundingClientRect();

  let targetScroll = container.scrollTop;

  if (block === "start") {
    // Math: current scroll + visual offset from container top
    targetScroll += elRect.top - containerRect.top;
  } else if (block === "center") {
    // Aligns the center of the element with the center of the container
    const elCenter = elRect.top + elRect.height / 2;
    const containerCenter = containerRect.top + containerRect.height / 2;
    targetScroll += elCenter - containerCenter;
  } else if (block === "nearest") {
    // Only scroll if element is outside the visible viewport
    if (elRect.top < containerRect.top) {
      targetScroll += elRect.top - containerRect.top; // Align top
    } else if (elRect.bottom > containerRect.bottom) {
      targetScroll += elRect.bottom - containerRect.bottom; // Align bottom
    }
  }

  // Clamp the calculated target to prevent overscroll
  const maxScroll = Math.max(0, container.scrollHeight - container.clientHeight);
  const clampedScrollTop = Math.min(Math.max(0, targetScroll), maxScroll);

  scrollTo(container, clampedScrollTop);
}
