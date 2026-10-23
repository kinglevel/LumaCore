// Pictures the show embeds (decisions §30 "12a2"): a client uploads a file
// into library/media/ (POST /api/media), the same `Import …` line anyone
// types puts it into a pool, GET /api/media/<sha256> serves it by hash, the
// profile's look reaches the client in /api/me, and the show saves as a
// zip of the ShowData tree (format 7 and on) with the picture beside its
// manifest.

import path from 'node:path';
import { expect, test } from '@playwright/test';
import { Server, zipEntries } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(); });
test.afterAll(async () => { await server.cleanup(); });

/** A PNG header of 4 × 3 (the size is read from it). */
function png(): Buffer {
  const b = Buffer.alloc(24);
  Buffer.from('\x89PNG\r\n\x1a\n\0\0\0\rIHDR', 'latin1').copy(b);
  b.writeUInt32BE(4, 16);
  b.writeUInt32BE(3, 20);
  return b;
}

test('upload, import, serve by hash and show the look in /api/me', async () => {
  const token = await server.login('Admin');
  const auth = { Cookie: `lumacore_token=${token}` };
  // Upload needs a login; the name is made safe.
  expect((await fetch(`${server.url}/api/media?name=x.png`, { method: 'POST', body: png() })).status).toBe(401);
  const up = await fetch(`${server.url}/api/media?name=${encodeURIComponent('../bg.png')}`, {
    method: 'POST', headers: auth, body: png(),
  });
  const saved = await up.json();
  expect(saved).toMatchObject({ ok: true, name: 'bg.png' });

  // The same line anyone types.
  let r = await server.command(`Import UserProfile Image "${saved.name}"`);
  expect(r.ok, r.message).toBe(true);
  r = await server.command('Set UserProfile LockScreen "Image" 1');
  expect(r.ok, r.message).toBe(true);
  r = await server.command('Set UserProfile Theme "Base" Night');
  expect(r.ok, r.message).toBe(true);

  const me = await (await fetch(`${server.url}/api/me`, { headers: auth })).json();
  expect(me.look.theme.preset).toBe('Night');
  const image = me.look.lock_screen.image;
  expect(image).toMatchObject({ ext: 'png', width: 4, height: 3 });

  // Served by hash, open (the lock screen shows it before a login), cached.
  const got = await fetch(`${server.url}/api/media/${image.sha256}`);
  expect(got.status).toBe(200);
  expect(got.headers.get('content-type')).toBe('image/png');
  expect(got.headers.get('cache-control')).toContain('immutable');
  expect(Buffer.from(await got.arrayBuffer()).equals(png())).toBe(true);
  expect((await fetch(`${server.url}/api/media/${'0'.repeat(64)}`)).status).toBe(404);
  expect((await fetch(`${server.url}/api/media/..%2Fx`)).status).toBe(404);
});

test('the show saves as a zip of the tree with the picture beside its manifest', async () => {
  const r = await server.command('SaveShow "media12a2"');
  expect(r.ok, r.message).toBe(true);
  // A `.luma` is a zip of the ShowData tree (decisions §30 "12a2"): the
  // manifest first, a folder per pool member, the picture beside its JSON.
  const names = zipEntries(path.join(server.dir, 'library/shows/media12a2.luma'));
  expect(names[0]).toBe('manifest.json');
  // Profile 1 is the locked template Default; Admin's own is 2 (decisions §30 "12c2").
  for (const entry of ['UserProfiles/2 Admin/Content/Images/1 bg/image.json', 'UserProfiles/2 Admin/Content/Images/1 bg/image.png']) {
    expect(names, entry).toContain(entry);
  }
  // Loads again.
  const back = await server.command('LoadShow "media12a2" /NoConfirmation');
  expect(back.ok, back.message).toBe(true);
});
