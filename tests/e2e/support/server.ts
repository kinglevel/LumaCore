// A LumaCore server for one spec file: `lumacore-server` in a fresh temp
// directory (its own library/, station id and users), on its own ports so it
// never meets a server you have running, with `--RootMonitor` so every sysmon
// line also lands on stderr.
//
// Ports: spec files run in parallel (one worker each, playwright.config.ts), so
// a server takes a free block of ten ports when it starts — the worker's index
// spreads the blocks (50000 + 300 per worker), a bind probe skips a taken one —
// and keeps it across a restart.

import { spawn, type ChildProcess } from 'node:child_process';
import { createSocket } from 'node:dgram';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { Socket, createServer } from 'node:net';
import { networkInterfaces, tmpdir } from 'node:os';
import path from 'node:path';
import { expect, type Page } from '@playwright/test';

export const REPO = path.resolve(__dirname, '../../..');
const BINARY = path.join(REPO, 'target/debug/lumacore-server');
const FRONTEND = path.join(REPO, 'crates/ui/lumacore-webui/web');

/** Can this process bind `port` right now (TCP, or UDP with `udp`)? */
function free(port: number, udp = false): Promise<boolean> {
  return new Promise(resolve => {
    if (udp) {
      const s = createSocket('udp4');
      s.once('error', () => resolve(false));
      s.bind(port, '127.0.0.1', () => s.close(() => resolve(true)));
    } else {
      const s = createServer();
      s.once('error', () => resolve(false));
      s.listen(port, '127.0.0.1', () => s.close(() => resolve(true)));
    }
  });
}

let nextBlock = 0;
/** A free block of ten ports for one server (see the header). */
async function freeBase(): Promise<number> {
  const worker = Number(process.env.TEST_PARALLEL_INDEX ?? 0);
  for (let i = 0; i < 30; i++) {
    const base = 50000 + worker * 300 + (nextBlock++ % 30) * 10;
    const ok = await Promise.all([free(base), free(base + 1), free(base + 5), free(base + 2, true), free(base + 3, true)]);
    if (ok.every(Boolean)) return base;
  }
  throw new Error('no free port block for a test server');
}

// A worker that dies (a timeout, a crash) must not leave servers behind.
const running = new Set<ChildProcess>();
process.once('exit', () => { for (const p of running) if (p.exitCode === null) p.kill('SIGKILL'); });

export class Server {
  readonly lines: string[] = [];
  private proc?: ChildProcess;
  /** The server's working directory (its `library/` lives here). */
  dir = '';
  /** The first port of its block (set by the first `start`). */
  base = 0;

  constructor() {}

  get http() { return this.base; }
  get shell() { return this.base + 1; }
  get cmd() { return this.base + 2; }
  get discovery() { return this.base + 3; }
  get stream() { return this.base + 5; }
  get url() { return `http://127.0.0.1:${this.http}`; }

  /** Start (again: same library). `extra`: more server arguments, e.g. `--clean-start`; `env`: more
   *  environment for the server (`LUMACORE_DRIVES_PROBE`: a folder whose subfolders are its drives). */
  async start(extra: string[] = [], env: Record<string, string> = {}): Promise<this> {
    if (!this.dir) this.dir = mkdtempSync(path.join(tmpdir(), 'lumacore-e2e-'));
    if (!this.base) this.base = await freeBase();
    this.proc = spawn(
      BINARY,
      [
        '--http-port', String(this.http),
        '--shell-port', String(this.shell),
        '--cmd-port', String(this.cmd),
        '--discovery-port', String(this.discovery),
        '--stream-port', String(this.stream),
        '--web-root', FRONTEND,
        '--RootMonitor',
        ...extra,
      ],
      { cwd: this.dir, stdio: ['ignore', 'pipe', 'pipe'], env: { ...process.env, ...env } },
    );
    running.add(this.proc);
    this.proc.once('exit', () => running.delete(this.proc!));
    const collect = (chunk: Buffer) => {
      for (const line of chunk.toString().split(/\r?\n/)) {
        if (!line.trim()) continue;
        this.lines.push(line);
        // E2E_ECHO=<regex>: print matching server lines while debugging.
        if (process.env.E2E_ECHO && new RegExp(process.env.E2E_ECHO).test(line)) console.log(`[${this.http}] ${line}`);
      }
    };
    this.proc.stdout!.on('data', collect);
    this.proc.stderr!.on('data', collect);
    const deadline = Date.now() + 20_000;
    while (Date.now() < deadline) {
      try {
        const r = await fetch(`${this.url}/api/station`);
        if (r.ok) return this;
      } catch {}
      await new Promise(r => setTimeout(r, 40));
    }
    throw new Error(`server on ${this.http} did not start:\n${this.lines.slice(-30).join('\n')}`);
  }

  /** Stop the process (the temp library stays until `cleanup`). SIGTERM is a
   *  clean shutdown the UIs are told about (decisions §23 "App ↔ local server"). */
  async stop() {
    await this.end('SIGTERM');
  }

  /** Lose the process without a word (SIGKILL), like a crash or a pulled cable. */
  async crash() {
    await this.end('SIGKILL');
  }

  private async end(signal: NodeJS.Signals) {
    const p = this.proc;
    if (!p || p.exitCode !== null) return;
    const gone = new Promise(r => p.once('exit', r));
    p.kill(signal);
    await Promise.race([gone, new Promise(r => setTimeout(r, 3000))]);
    if (p.exitCode === null) p.kill('SIGKILL');
  }

  async cleanup() {
    await this.stop();
    if (this.dir) rmSync(this.dir, { recursive: true, force: true });
  }

  /** Position in the output; `waitSysmon` only looks after it. */
  mark() { return this.lines.length; }

  /** Wait for a sysmon line matching `re` after `from`. */
  async waitSysmon(re: RegExp, from = 0, timeout = 8000): Promise<string> {
    const deadline = Date.now() + timeout;
    while (Date.now() < deadline) {
      const hit = this.lines.slice(from).find(l => re.test(l));
      if (hit) return hit;
      await new Promise(r => setTimeout(r, 50));
    }
    throw new Error(`no sysmon line ${re} — last lines:\n${this.lines.slice(-25).join('\n')}`);
  }

  /** Sysmon lines matching `re` after `from` (to check something did NOT happen). */
  sysmonSince(re: RegExp, from = 0) { return this.lines.slice(from).filter(l => re.test(l)); }

  /** Run `line` as `user` over the UDP command port (like `lumacore-cli`);
   *  works whatever the page shows (a menu open, a pop-up). The reply. */
  async command(line: string, user = 'Admin'): Promise<{ ok: boolean; message: string }> {
    const dgram = await import('node:dgram');
    const udp = dgram.createSocket('udp4');
    try {
      return await new Promise((resolve, reject) => {
        const timer = setTimeout(() => reject(new Error(`no reply to ${line}`)), 5000);
        udp.once('message', msg => { clearTimeout(timer); resolve(JSON.parse(msg.toString())); });
        udp.send(JSON.stringify({ kind: 'command', user, cmd: line }), this.cmd, '127.0.0.1');
      });
    } finally {
      udp.close();
    }
  }

  /** Open the station's page already logged in (decisions §30 "12a3": Admin's
   *  password is always `admin`, so nothing logs in by itself): the token goes
   *  into the page's cookie first. */
  async open(page: Page, path = '/', user = 'Admin', password?: string): Promise<void> {
    const token = await this.login(user, password);
    await page.context().addCookies([{ name: 'lumacore_token', value: token, url: this.url }]);
    await page.goto(this.url + path);
  }

  /** Log in over HTTP like the app does: the token (Admin: `admin`). The login is finished
   *  with the Meow License's consent (decisions 0290) unless `consent` is false: the token
   *  opens nothing until then. */
  async login(user: string, password?: string, host = '127.0.0.1', consent = true): Promise<string> {
    if (password === undefined && user.toLowerCase() === 'admin') password = 'admin';
    const r = await fetch(`http://${host}:${this.http}/api/login`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ user, password }),
    });
    const d = await r.json();
    if (!r.ok) throw new Error(`login ${user}: ${d.message}`);
    if (consent) await this.consent(d.token, host);
    return d.token;
  }

  /** Accept the Meow License for a token's login (decisions 0290), the way any client does: a
   *  shell logs in with the token, runs `Consent` and leaves. */
  async consent(token: string, host = '127.0.0.1'): Promise<void> {
    const sock = new Socket();
    await new Promise<void>((resolve, reject) => {
      sock.once('error', reject);
      sock.connect(this.shell, host, () => resolve());
    });
    try {
      const statuses: any[] = [];
      let buf = '';
      sock.on('data', d => {
        buf += d.toString();
        let i;
        while ((i = buf.indexOf('\n')) >= 0) {
          const packet = buf.slice(0, i);
          buf = buf.slice(i + 1);
          try {
            const p = JSON.parse(packet);
            if (p.kind === 'status') statuses.push(p.status);
            if (p.kind === 'error') statuses.push({ error: p.message });
          } catch {}
        }
      });
      const until = async (ok: (s: any) => boolean, what: string) => {
        const deadline = Date.now() + 8000;
        while (Date.now() < deadline) {
          const err = statuses.find(s => s.error);
          if (err) throw new Error(`consent: ${err.error}`);
          if (statuses.some(ok)) return;
          await new Promise(r => setTimeout(r, 20));
        }
        throw new Error(`consent: ${what}`);
      };
      const asks = (s: any) => (s.dialogs || []).some((d: any) => d.topic === 'consent');
      sock.write(JSON.stringify({ kind: 'hello', user: '', token, width: 80, height: 24, protocol: CLIENT_PROTOCOL }) + '\n');
      await until(asks, 'the Meow License never asked');
      statuses.length = 0;
      sock.write(JSON.stringify({ kind: 'command', cmd: 'Consent' }) + '\n');
      await until(s => !asks(s), 'the consent never finished the login');
      sock.write(JSON.stringify({ kind: 'disconnect' }) + '\n');
    } finally {
      sock.destroy();
    }
  }
}

/** `lumacore_wire::CLIENT_PROTOCOL`: a station refuses any other (E-WIRE-001). Read from the
 *  crate's source, so a bump never leaves the test clients behind. */
const CLIENT_PROTOCOL = Number(
  /pub const CLIENT_PROTOCOL: u32 = (\d+);/.exec(readFileSync(path.join(REPO, 'crates/foundation/lumacore-wire/src/lib.rs'), 'utf8'))?.[1],
);

/** A sysmon viewer like `lumacore-cli --meownitor`: TCP shell port, MonitorHello. */
export class Monitor {
  readonly lines: string[] = [];
  private sock = new Socket();
  private buf = '';

  static async connect(server: Server): Promise<Monitor> {
    const m = new Monitor();
    await new Promise<void>((resolve, reject) => {
      m.sock.once('error', reject);
      m.sock.connect(server.shell, '127.0.0.1', () => resolve());
    });
    m.sock.on('data', d => {
      m.buf += d.toString();
      let i;
      while ((i = m.buf.indexOf('\n')) >= 0) {
        const packet = m.buf.slice(0, i);
        m.buf = m.buf.slice(i + 1);
        try {
          const p = JSON.parse(packet);
          if (p.kind === 'frame') for (const l of String(p.ansi).split(/\r?\n/)) if (l.trim()) m.lines.push(l);
        } catch {}
      }
    });
    m.sock.write(JSON.stringify({ kind: 'monitor_hello', protocol: CLIENT_PROTOCOL }) + '\n');
    return m;
  }

  async wait(re: RegExp, timeout = 8000): Promise<string> {
    const deadline = Date.now() + timeout;
    while (Date.now() < deadline) {
      const hit = this.lines.find(l => re.test(l));
      if (hit) return hit;
      await new Promise(r => setTimeout(r, 50));
    }
    throw new Error(`monitor never showed ${re} — got:\n${this.lines.slice(-20).join('\n')}`);
  }

  close() { this.sock.destroy(); }
}

/** A terminal client like `lumacore-cli` (TCP shell port, Hello): the
 *  station draws its TUI and sends ANSI frames; they are drawn into the
 *  vendored xterm.js in `screen` (a page of its own), so a test reads the
 *  terminal exactly as a user would see it. */
export class Cli {
  private sock = new Socket();
  private buf = '';
  constructor(readonly screen: Page, readonly cols: number, readonly rows: number) {}

  /** The newest status the station sent. */
  status: any = null;

  static async connect(server: Server, screen: Page, user: string, password?: string, cols = 120, rows = 40, consent = true): Promise<Cli> {
    const c = new Cli(screen, cols, rows);
    await screen.goto(`${server.url}/vendor/xterm.css`).catch(() => {});
    await screen.setContent(`<!doctype html><div id="t"></div>`);
    await screen.addScriptTag({ url: `${server.url}/vendor/xterm.js` });
    await screen.evaluate(([cols, rows]) => {
      const T = (window as any).Terminal;
      const term = new T({ cols, rows, allowProposedApi: true });
      term.open(document.getElementById('t'));
      (window as any).term = term;
    }, [cols, rows]);
    await new Promise<void>((resolve, reject) => {
      c.sock.once('error', reject);
      c.sock.connect(server.shell, '127.0.0.1', () => resolve());
    });
    c.sock.on('data', d => {
      c.buf += d.toString();
      let i;
      while ((i = c.buf.indexOf('\n')) >= 0) {
        const packet = c.buf.slice(0, i);
        c.buf = c.buf.slice(i + 1);
        try {
          const p = JSON.parse(packet);
          if (p.kind === 'frame' && p.view == null) void screen.evaluate(a => (window as any).term.write(a), String(p.ansi)).catch(() => {});
          if (p.kind === 'status') c.status = p.status;
        } catch {}
      }
    });
    c.send({ kind: 'hello', user, width: cols, height: rows, ...(password ? { password } : {}), protocol: CLIENT_PROTOCOL });
    // The login asks for the Meow License (decisions 0290): press its first button, like a user.
    if (consent) {
      const asks = () => (c.status?.dialogs || []).some((d: any) => d.topic === 'consent');
      await expect.poll(asks, { message: 'the Meow License never asked', timeout: 10_000 }).toBe(true);
      c.key({ Char: '1' });
      await expect.poll(asks, { message: 'the consent never finished the login', timeout: 10_000 }).toBe(false);
    }
    return c;
  }

  send(packet: unknown) { this.sock.write(JSON.stringify(packet) + '\n'); }
  key(code: unknown, modifiers = 0) { this.send({ kind: 'key', key: { code, modifiers } }); }
  type(text: string) { for (const ch of text) this.key({ Char: ch }); }

  /** The terminal's screen as text, one string per row. */
  text(): Promise<string[]> {
    return this.screen.evaluate(() => {
      const b = (window as any).term.buffer.active;
      const out: string[] = [];
      for (let y = 0; y < b.length; y++) out.push(b.getLine(y)?.translateToString(true) ?? '');
      return out.slice(-((window as any).term.rows));
    });
  }

  /** The titles in the TUI's suggestion box (empty when none is open). */
  async suggestions(): Promise<string[]> {
    const lines = await this.text();
    const top = lines.findIndex(l => l.includes(' Suggestions ') && l.includes('Tab takes'));
    if (top < 0) return [];
    const out: string[] = [];
    for (const line of lines.slice(top + 1)) {
      const at = line.indexOf('│');
      if (at < 0) break;
      const title = [...line.slice(at + 1)].slice(3, 27).join('').trimEnd();
      if (!title.trim()) break;
      out.push(title);
    }
    return out;
  }

  close() { this.sock.destroy(); }
}

/** Open a WebSocket to `url` inside `page` (Node 20 has no global WebSocket;
 *  the browser's is the one a client uses anyway), send `first` once it is
 *  open, and collect the text messages (parsed as JSON where they are JSON,
 *  a message that is an array is flattened) until `done(messages)` is true
 *  (`done` runs in the page: a self-contained function, no closure).
 *  `about:blank` has no origin, so the page is taken to the server first. */
export async function wsMessages(page: Page, url: string, first: unknown, done: (messages: any[]) => boolean, timeout = 5000): Promise<any[]> {
  if (page.url() === 'about:blank') await page.goto(url.replace(/^ws/, 'http').replace(/\/ws\/.*$/, '/api/station'));
  return page.evaluate(([url, first, timeout, doneSrc]) => new Promise<any[]>((resolve, reject) => {
    const done = new Function(`return (${doneSrc})`)() as (m: any[]) => boolean;
    const ws = new WebSocket(url);
    const got: any[] = [];
    const timer = setTimeout(() => { ws.close(); reject(new Error(`no answer in ${timeout} ms — got ${JSON.stringify(got).slice(0, 500)}`)); }, timeout);
    ws.onopen = () => { if (first !== null) ws.send(typeof first === 'string' ? first : JSON.stringify(first)); };
    ws.onmessage = e => {
      if (typeof e.data !== 'string') return;
      let m: any = e.data;
      try { m = JSON.parse(e.data); } catch {}
      got.push(...(Array.isArray(m) ? m : [m]));
      if (done(got)) { clearTimeout(timer); ws.close(); resolve(got); }
    };
    ws.onerror = () => { clearTimeout(timer); reject(new Error('ws error')); };
  }), [url, first, timeout, done.toString()] as const);
}

/** The entry names of a zip file (a `.luma` is a zip of the ShowData tree,
 *  decisions §30 "12a2"), read from its central directory. */
export function zipEntries(file: string): string[] {
  const b = readFileSync(file);
  // End of central directory record (0x06054b50), searched back over the comment.
  let eocd = -1;
  for (let i = b.length - 22; i >= Math.max(0, b.length - 22 - 65535); i--) if (b.readUInt32LE(i) === 0x06054b50) { eocd = i; break; }
  if (eocd < 0) throw new Error(`${file}: not a zip`);
  const count = b.readUInt16LE(eocd + 10);
  let at = b.readUInt32LE(eocd + 16);
  const names: string[] = [];
  for (let n = 0; n < count; n++) {
    if (b.readUInt32LE(at) !== 0x02014b50) throw new Error(`${file}: bad central directory entry ${n}`);
    const nameLen = b.readUInt16LE(at + 28), extraLen = b.readUInt16LE(at + 30), commentLen = b.readUInt16LE(at + 32);
    names.push(b.toString('utf8', at + 46, at + 46 + nameLen));
    at += 46 + nameLen + extraLen + commentLen;
  }
  return names;
}

/** This machine's LAN address (a "remote" client of its own server), if any. */
export function lanAddress(): string | undefined {
  for (const list of Object.values(networkInterfaces()))
    for (const a of list ?? []) if (a.family === 'IPv4' && !a.internal) return a.address;
  return undefined;
}

// ── Page helpers ────────────────────────────────────────────────────────────

/** The shell WebSocket is up: the top bar's mark shows a connected state (decisions top-bar-status-strip).
 *  A login still waiting for the Meow License is answered first, like a user (decisions 0290). */
export async function connected(page: Page) {
  await expect(page.locator('#tb-strip .lc-ss-mark')).toHaveAttribute('data-state', /^(ok|member|busy|idle)$/, { timeout: 15_000 });
  await answerConsent(page);
}

/** The Meow License's dialog (decisions 0290), when it shows. */
export const consentDialog = (page: Page) => page.locator('[data-kit="dialog"] .lc-dlg[aria-label="Meow License"]');

/** A login just made on this page: the Meow License asks (decisions 0290); press "I consent". */
export async function giveConsent(page: Page) {
  await expect(consentDialog(page)).toBeVisible({ timeout: 15_000 });
  await answerConsent(page);
}

/** Every login asks for the Meow License (decisions 0290): when this page's login waits for it,
 *  press "I consent" like a user and wait for the login to finish. Nothing waits: nothing to do.
 *  `wait`: how long to look for a dialog that may still be on its way (an in-shell Login). */
export async function answerConsent(page: Page, wait = 0) {
  const waiting = () => page.evaluate('typeof consenting !== "undefined" && consenting !== null').catch(() => false);
  const shows = async () => (await consentDialog(page).count()) > 0;
  const deadline = Date.now() + wait;
  // The page knows it waits (its own login), or the dialog may still come (a line's Login).
  await expect
    .poll(async () => (await shows()) || !(await waiting()) && Date.now() >= deadline, { timeout: 15_000 })
    .toBe(true);
  if (!(await shows())) return;
  await consentDialog(page).locator('.lc-dlg-btn[data-primary="1"]').click();
  await expect(consentDialog(page)).toHaveCount(0);
  await expect.poll(waiting, { timeout: 15_000 }).toBe(false);
}

/** Keyboard mode (KBD): letters type text. In PRG they are shortcuts
 *  (`Logout` would become `Group Off Update Thru`). Clicks the mode button on
 *  the command line bar like a user; the mode is the server's, so it sticks. */
export async function kbdMode(page: Page) {
  const mode = page.locator('#cb-mode .cb-text');
  // The button says KBD in the markup until the server's first status.
  await page.waitForFunction('typeof barStatus !== "undefined" && barStatus !== null', null, { timeout: 10_000 });
  // A phone keeps the mode in the bar's ⋯ menu (decisions §30 "12c2").
  const onBar = await page.locator('#cb-mode').isVisible();
  for (let i = 0; i < 3 && (await mode.textContent())!.trim() !== 'KBD'; i++) {
    const before = (await mode.textContent())!.trim();
    if (onBar) await page.locator('#cb-mode').click();
    else {
      await page.locator('#cb-more').click();
      await page.locator('#ctx .ci', { hasText: 'Shortcut mode' }).click();
    }
    await expect(mode).not.toHaveText(before);
  }
  await expect(mode).toHaveText('KBD');
  // Focus back on the page body (the button keeps focus otherwise).
  await page.locator('#cb-line').click();
}

/** Type a command on the command line, like a user at the keyboard. */
export async function cmd(page: Page, text: string) {
  await page.keyboard.type(text, { delay: 5 });
  await page.keyboard.press('Enter');
  // Ran: the command line bar is an empty prompt again (or the page left the
  // session, e.g. Logout).
  await expect
    .poll(async () => {
      if (await page.locator('.mo-overlay.open, [data-kit="dialog"], luma-surface').count()) return true;
      const line = (await page.locator('#cb-line .xterm-rows').innerText().catch(() => '')).trim();
      return /[>]$/.test(line);
    }, { message: `command did not run: ${text}`, timeout: 10_000 })
    .toBe(true);
  // A Login asks for the Meow License (decisions 0290) unless it asked for a password first.
  if (/^login\b/i.test(text.trim()) && !(await page.locator('[data-kit="dialog"]').count())) await answerConsent(page, 3000);
  else if (await consentDialog(page).count()) await answerConsent(page);
}

/** Everything the terminals and the command line bar show right now. */
export async function screenText(page: Page): Promise<string> {
  return (await page.locator('.xterm-rows').allInnerTexts()).join('\n');
}

/** The command line bar prompt shows `user`. */
export async function promptUser(page: Page, user: string) {
  await expect(page.locator('#cb-line .xterm-rows')).toContainText(`[${user}@`, { timeout: 10_000 });
  // A user switched to accepts the Meow License first (decisions 0290).
  await answerConsent(page);
}

// ── UI kit helpers (decisions §30) ──────────────────────────────────────────

/** Tap `target` like a finger (a click without touch): first check that the
 *  element is really what sits under its centre — nothing covers it (an exact
 *  hit test). */
export async function tap(page: Page, target: import('@playwright/test').Locator) {
  await target.scrollIntoViewIfNeeded();
  const box = await target.boundingBox();
  if (!box) throw new Error('tap: the element has no box (hidden?)');
  const x = box.x + box.width / 2, y = box.y + box.height / 2;
  const hit = await target.evaluate((el, [px, py]) => {
    const top = document.elementFromPoint(px, py);
    if (top && (top === el || el.contains(top))) return '';
    return `${top?.tagName.toLowerCase() ?? 'nothing'}${top?.className ? `.${String(top.className).replace(/\s+/g, '.')}` : ''}`;
  }, [x, y]);
  expect(hit, `tap: something covers the element: ${hit}`).toBe('');
  const touch = await page.evaluate(() => matchMedia('(any-pointer: coarse)').matches);
  if (touch) await page.touchscreen.tap(x, y);
  else await page.mouse.click(x, y);
}

/** The element's box, polled: a feed redraw can replace the node between
 *  "visible" and `boundingBox()` (null), so ask again until it is there. */
export async function box(target: import('@playwright/test').Locator) {
  let b = await target.boundingBox();
  for (let i = 0; !b && i < 40; i++) { await new Promise(r => setTimeout(r, 50)); b = await target.boundingBox(); }
  if (!b) throw new Error('box: the element has no box (hidden or gone?)');
  return b;
}

/** Hold `target` still for `ms` (default: past the 500 ms hold time). */
// 900 ms clears the kit's longest hold (the claim button's 600 ms) with room for a loaded WebKit worker.
export async function hold(page: Page, target: import('@playwright/test').Locator, ms = 900) {
  await target.scrollIntoViewIfNeeded();
  const box = await target.boundingBox();
  if (!box) throw new Error('hold: the element has no box');
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.waitForTimeout(ms);
  await page.mouse.up();
}

/** The station's dialog on top (decisions §30 "12b": every question is the
 *  kit's standard dialog). */
export function dialog(page: Page) {
  return page.locator('[data-kit="dialog"]:not([hidden])').last();
}

/** Press the dialog's button labelled `label`. */
export async function answer(page: Page, label: string | RegExp) {
  await dialog(page).locator('.lc-dlg-btn', { hasText: label }).click();
}
