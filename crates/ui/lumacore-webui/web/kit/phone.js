// The phone rule (decisions §37 2): a screen whose short side is under 500 px is a phone — 390 ×
// 844 upright and 844 × 390 on its side; a 768 × 1024 tablet is not one. Never the width alone.
//
//   phoneMode() → 'up' (upright) · 'side' (on its side) · '' (not a phone)
//   onPhoneMode(fn) → off(): fn(mode) whenever the mode changes (a turn, a resize)

const SHORT = 500;

export function phoneMode() {
  const w = window.innerWidth, h = window.innerHeight;
  if (!w || !h || Math.min(w, h) >= SHORT) return '';
  return w > h ? 'side' : 'up';
}

export function onPhoneMode(fn) {
  let now = phoneMode();
  const check = () => { const m = phoneMode(); if (m !== now) { now = m; fn(m); } };
  window.addEventListener('resize', check);
  return () => window.removeEventListener('resize', check);
}
