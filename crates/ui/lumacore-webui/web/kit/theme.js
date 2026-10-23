// Theme presets and the colours a canvas or xterm needs as concrete values
// (decisions §30). The colours themselves live in tokens.css; this module only
// switches presets and resolves tokens. Part of the theme module, so it may
// build colour strings (tidy allowlist).

export const PRESETS = ['Dark', 'Daylight', 'Night'];
const root = document.documentElement;

/** The active preset on this page. */
export function preset() { return root.dataset.theme || 'Dark'; }

/** Show `name` on this page (Dark = no attribute). Fires `lc-theme`. */
export function setPreset(name) {
  if (!PRESETS.includes(name) || name === 'Dark') delete root.dataset.theme;
  else root.dataset.theme = name;
  window.dispatchEvent(new CustomEvent('lc-theme', { detail: { preset: preset() } }));
}

// One 1×1 canvas resolves any CSS colour (color-mix() included) to RGBA.
let probe = null, probeCtx = null, probeEl = null;
function rgba(css) {
  if (!probe) {
    probe = document.createElement('canvas');
    probe.width = probe.height = 1;
    probeCtx = probe.getContext('2d', { willReadFrequently: true });
    probeEl = document.createElement('span');
    probeEl.style.display = 'none';
    document.body.appendChild(probeEl);
  }
  probeEl.style.color = '';
  probeEl.style.color = css;
  const computed = getComputedStyle(probeEl).color;
  probeCtx.clearRect(0, 0, 1, 1);
  probeCtx.fillStyle = computed;
  probeCtx.fillRect(0, 0, 1, 1);
  const [r, g, b, a] = probeCtx.getImageData(0, 0, 1, 1).data;
  return { r, g, b, a: a / 255 };
}
const hex = ({ r, g, b }) => '#' + [r, g, b].map(v => v.toString(16).padStart(2, '0')).join('');

/** A token (`bg`, `accent`, `gap`, …) as `#rrggbb`. */
export function token(name) { return hex(rgba(`var(--lc-${name})`)); }

/** A token blended toward white (`amount` 0–1) as `#rrggbb`. */
function lift(name, amount) {
  const c = rgba(`var(--lc-${name})`);
  const f = v => Math.round(v + (255 - v) * amount);
  return hex({ r: f(c.r), g: f(c.g), b: f(c.b) });
}

/** xterm's theme from the tokens: the terminal follows the user's theme. */
export function terminalTheme({ background = 'gap' } = {}) {
  return {
    background: token(background), foreground: token('text'), cursor: token('accent'),
    cursorAccent: token(background), selectionBackground: token('selected'),
    black: token('gap'), red: token('alert'), green: token('ok'), yellow: lift('accent', 0.35),
    blue: token('info'), magenta: token('programmer'), cyan: token('park'), white: token('text'),
    brightBlack: token('text-dim'), brightRed: lift('alert', 0.25), brightGreen: lift('ok', 0.25),
    brightYellow: lift('accent', 0.55), brightBlue: lift('info', 0.25), brightMagenta: lift('programmer', 0.25),
    brightCyan: lift('park', 0.25), brightWhite: token('strong'),
  };
}

/** A fully transparent colour (xterm cursors that should not draw). */
export const TRANSPARENT = 'rgba(0,0,0,0)';

// ── The profile's look (decisions §30 "12a2" / "12a3") ──────────────────────
// A preset, five adjusters and only the changed palette colours. The adjusted
// palette is written as inline --lc-* values on <html>; the derived roles in
// tokens.css follow by themselves. The normal theme (Meow+F9, the station's
// ThemeOverride) shows the factory preset and nothing of the profile.

const PALETTE_KEYS = {
  background: 'bg', surface: 'surface', raised: 'raised', line: 'line', text: 'text', text_dim: 'text-dim',
  accent: 'accent', info: 'info', ok: 'ok', warn: 'warn', alert: 'alert', programmer: 'programmer',
  blind: 'blind', park: 'park',
};

function toHsl({ r, g, b }) {
  r /= 255; g /= 255; b /= 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b), l = (max + min) / 2;
  if (max === min) return { h: 0, s: 0, l };
  const d = max - min, s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
  const h = max === r ? (g - b) / d + (g < b ? 6 : 0) : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return { h: h * 60, s, l };
}
function fromHsl({ h, s, l }) {
  const k = n => (n + h / 30) % 12, a = s * Math.min(l, 1 - l);
  const f = n => l - a * Math.max(-1, Math.min(k(n) - 3, 9 - k(n), 1));
  return { r: Math.round(f(0) * 255), g: Math.round(f(8) * 255), b: Math.round(f(4) * 255) };
}
const clamp = (v, lo = 0, hi = 1) => Math.max(lo, Math.min(hi, v));

/** The neutral greys the background adjusters move (archive §30 "12g"). */
const NEUTRAL = new Set(['background', 'surface', 'raised', 'line']);

/** One colour through the adjusters (−100…100; accent hue −180…180°; a neutral: the background
 *  hue 0…360° and saturation 0…100 blend it toward that hue at its own lightness, first). */
function adjust(rgb, a, isAccent, isNeutral = false) {
  let { h, s, l } = toHsl(rgb);
  if (isNeutral && a.background_saturation) {
    const t = a.background_saturation / 100 * 0.55;
    const to = fromHsl({ h: a.background_hue || 0, s: 1, l });
    ({ h, s, l } = toHsl({ r: Math.round(rgb.r + (to.r - rgb.r) * t), g: Math.round(rgb.g + (to.g - rgb.g) * t), b: Math.round(rgb.b + (to.b - rgb.b) * t) }));
  }
  l = clamp(l + (a.brightness || 0) / 250);
  l = clamp(0.5 + (l - 0.5) * (1 + (a.contrast || 0) / 100));
  s = clamp(s * (1 + (a.saturation || 0) / 100));
  if (isAccent && a.accent_hue) h = (h + a.accent_hue + 360) % 360;
  return fromHsl({ h, s, l });
}
/** Whether any adjuster changes a colour (the background hue alone does not). */
export const adjusted = a => !!(a && (a.brightness || a.contrast || a.saturation || a.accent_hue || a.background_saturation));

/** Show a profile's `look` ({theme, lock_screen} from /api/me) — or, with
 *  `normal`, the factory preset (`override` names it; default Dark). */
export function applyLook(look, { normal = false, override = null } = {}) {
  for (const tok of Object.values(PALETTE_KEYS)) root.style.removeProperty(`--lc-${tok}`);
  const theme = look?.theme;
  if (normal || !theme) { setPreset(normal ? (override || 'Dark') : 'Dark'); return; }
  setPreset(theme.preset || 'Dark');
  const a = theme.adjust || {}, palette = theme.palette || {};
  const changed = adjusted(a);
  for (const [key, tok] of Object.entries(PALETTE_KEYS)) {
    const own = palette[key];
    if (!own && !changed) continue;
    const base = rgba(own || `var(--lc-${tok})`);
    root.style.setProperty(`--lc-${tok}`, hex(changed ? adjust(base, a, key === 'accent', NEUTRAL.has(key)) : base));
  }
  window.dispatchEvent(new CustomEvent('lc-theme', { detail: { preset: preset() } }));
}
