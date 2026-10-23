// The user's clipboard: a GUID, a path, an object target, a name or a Set line the
// NAV inspector copies (the `copy_text` UI event on the page's shell) goes on the
// clipboard of the computer this page runs on, and a toast says "Copied <label>".
// Never the station's object clipboard (Copy / Cut / Paste of show objects).
//
//   copyText(label, text, { host })   host = window.LumaHost (the desktop app) or null
//   appCopied(ok)                     the app's answer (lumacoreApp.copied)
//
// The ways, in order (docs/dev/ui-kit/mockups/2026-10-23-nav-mode/02-inspector.html,
// "Copy on every client"): in the app, its IPC (`copy { text }`, written natively);
// the browser's clipboard (https or localhost); the older copy command, which works
// while the key press still counts as the user's; else a toast that holds the text
// selected, ready for Ctrl+C.

import { toast } from './toast.js';

let waiting = null; // { label, text }: handed to the app, waiting for its answer

const said = label => toast(`Copied ${label}`);

export async function copyText(label, text, { host = null } = {}) {
  if (!text) return;
  if (host?.copy) {
    waiting = { label, text };
    host.copy(text);
    return;
  }
  await inBrowser(label, text);
}

/** The app wrote the clipboard (ok) or couldn't (the page tries its own ways). */
export function appCopied(ok) {
  const w = waiting;
  waiting = null;
  if (!w) return;
  if (ok) said(w.label);
  else inBrowser(w.label, w.text);
}

async function inBrowser(label, text) {
  if (window.isSecureContext && navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      return said(label);
    } catch (_) { /* refused: the older way */ }
  }
  if (byCommand(text)) return said(label);
  toast(`Copy ${label}: Ctrl+C`, { hold: text, onCopy: () => said(label) });
}

/** The older copy command on a hidden field; the focus and its selection go back. */
function byCommand(text) {
  const had = document.activeElement;
  const range = had && 'selectionStart' in had ? [had.selectionStart, had.selectionEnd] : null;
  const field = document.createElement('textarea');
  field.value = text;
  field.readOnly = true;
  field.setAttribute('aria-hidden', 'true');
  field.style.cssText = 'position:fixed;left:-9999px;top:0;opacity:0';
  document.body.append(field);
  field.select();
  let ok = false;
  try { ok = document.execCommand('copy'); } catch (_) { ok = false; }
  field.remove();
  if (had?.isConnected) {
    had.focus({ preventScroll: true });
    if (range) try { had.setSelectionRange(range[0], range[1]); } catch (_) { /* not a text field */ }
  }
  return ok;
}
