// The multiplatform rules (decisions §30 "12c2"), measured in the page:
//   - no horizontal page scroll;
//   - nothing visible outside the viewport (after the clipping of scroll
//     boxes: a list that scrolls sideways inside its window is fine);
//   - every overlay box inside the viewport;
//   - no visible text under 12 px;
//   - on touch screens every visible interactive element ≥ 44 × 44 px
//     (what is not tappable — pointer-events: none, a miniature's tiny cells
//     — doesn't count; a built-in keyboard key keeps the OS keyboards' width
//     and is checked for its height only).
// `fitIssues(page, label, touch)` returns the problems as readable lines.

import type { Page } from '@playwright/test';

export interface FitOptions { touch: boolean }

/** Runs in the page (serialised by Playwright). */
function measure({ touch }: FitOptions): string[] {
  const W = window.innerWidth, H = window.innerHeight;
  const out: string[] = [];
  const name = (el: Element) => {
    const h = el as HTMLElement;
    const cls = typeof h.className === 'string' && h.className.trim() ? '.' + h.className.trim().split(/\s+/).slice(0, 2).join('.') : '';
    const text = (h.innerText || h.getAttribute('aria-label') || h.getAttribute('title') || '').replace(/\s+/g, ' ').trim().slice(0, 30);
    return `${el.tagName.toLowerCase()}${h.id ? '#' + h.id : ''}${cls}${text ? ` "${text}"` : ''}`;
  };
  const styles = new Map<Element, CSSStyleDeclaration>();
  const style = (e: Element) => { let s = styles.get(e); if (!s) { s = getComputedStyle(e); styles.set(e, s); } return s; };
  const shownMemo = new Map<Element, boolean>();
  const shown = (el: Element | null): boolean => {
    if (!el) return true;
    const known = shownMemo.get(el);
    if (known !== undefined) return known;
    const cs = style(el);
    const v = !(cs.display === 'none' || cs.visibility === 'hidden' || Number(cs.opacity) < 0.05
      || el.getAttribute('aria-hidden') === 'true' || el.id === 'lc-sprite') && shown(el.parentElement);
    shownMemo.set(el, v);
    return v;
  };
  // The part of `el` that can be seen: its box cut by every scrolling or
  // clipping ancestor (fixed elements stop at the viewport).
  const clipped = (el: Element) => {
    const r = el.getBoundingClientRect();
    let x0 = r.left, y0 = r.top, x1 = r.right, y1 = r.bottom;
    if (style(el).position === 'fixed') return { x0, y0, x1, y1, w: x1 - x0, h: y1 - y0, r };
    for (let e = el.parentElement; e && e !== document.documentElement; e = e.parentElement) {
      const cs = style(e);
      if (cs.overflowX !== 'visible' || cs.overflowY !== 'visible' || cs.contain.includes('paint')) {
        const p = e.getBoundingClientRect();
        if (cs.overflowX !== 'visible') { x0 = Math.max(x0, p.left); x1 = Math.min(x1, p.right); }
        if (cs.overflowY !== 'visible') { y0 = Math.max(y0, p.top); y1 = Math.min(y1, p.bottom); }
      }
      if (cs.position === 'fixed') break;
    }
    return { x0, y0, x1, y1, w: x1 - x0, h: y1 - y0, r };
  };

  if (document.documentElement.scrollWidth > W + 1) out.push(`horizontal page scroll: ${document.documentElement.scrollWidth} > ${W}`);

  const all = [...document.body.querySelectorAll('*')].filter(el => !(el instanceof SVGElement) && el.tagName !== 'LUMA-ICON');
  const visible = new Set<Element>();
  for (const el of all) {
    const c = clipped(el);
    if (c.w <= 0.5 || c.h <= 0.5) continue;
    if (!shown(el)) continue;
    visible.add(el);
  }

  // Outside the viewport: report the outermost element only.
  for (const el of visible) {
    const c = clipped(el);
    const outside = c.x0 < -1 || c.y0 < -1 || c.x1 > W + 1 || c.y1 > H + 1;
    if (!outside) continue;
    const p = el.parentElement;
    if (p && visible.has(p)) {
      const pc = clipped(p);
      if (pc.x0 < -1 || pc.y0 < -1 || pc.x1 > W + 1 || pc.y1 > H + 1) continue;
    }
    out.push(`outside the viewport ${Math.round(c.x0)},${Math.round(c.y0)}–${Math.round(c.x1)},${Math.round(c.y1)} (${W}×${H}): ${name(el)}`);
  }

  // Overlay boxes: inside the viewport (they scroll inside, never the page).
  for (const el of document.querySelectorAll('.lc-box, .lc-dlg, .lc-pick, .lc-kp, .lc-set, .lc-setup, .lc-card, #ctx:not([hidden]), luma-suggest:not([hidden]), #history-pop:not([hidden]), .pe-box, .lc-sg-inline')) {
    if (!visible.has(el)) continue;
    const r = el.getBoundingClientRect();
    if (r.left < -1 || r.top < -1 || r.right > W + 1 || r.bottom > H + 1)
      out.push(`overlay larger than the screen ${Math.round(r.width)}×${Math.round(r.height)} at ${Math.round(r.left)},${Math.round(r.top)}: ${name(el)}`);
  }

  // Text under 12 px.
  for (const el of visible) {
    const own = [...el.childNodes].some(n => n.nodeType === 3 && (n.textContent || '').trim());
    if (!own || el.closest('.xterm')) continue;
    const fs = parseFloat(getComputedStyle(el).fontSize);
    if (fs < 11.95) out.push(`text ${fs}px: ${name(el)}`);
  }

  // Touch targets.
  if (touch) {
    const INTERACTIVE = 'button, a[href], input:not([type="hidden"]), select, textarea, [role="button"], [role="tab"], [role="menuitem"], [role="switch"], [onclick], [data-key], .ptab, .tab-dot, .gpbtn, .vbtn, .lc-wb-item, .lc-pick-row, .bk-row, .lc-sg-row';
    for (const el of document.querySelectorAll(INTERACTIVE)) {
      if (!visible.has(el) || (el as HTMLButtonElement).disabled) continue;
      if (getComputedStyle(el).pointerEvents === 'none') continue;
      if (el.closest('.xterm')) continue; // xterm's own helper textarea
      const r = el.getBoundingClientRect();
      const key = el.classList.contains('lc-kb-key');
      if ((!key && r.width < 43.5) || r.height < 43.5) out.push(`touch target ${Math.round(r.width)}×${Math.round(r.height)}: ${name(el)}`);
    }
  }
  return [...new Set(out)];
}

export async function fitIssues(page: Page, label: string, opts: FitOptions): Promise<string[]> {
  await page.waitForTimeout(250);
  const issues = await page.evaluate(measure, opts);
  return issues.map(i => `[${label}] ${i}`);
}
