import test from 'node:test';
import assert from 'node:assert/strict';
import { ControlCenter } from './controlCenter.ts';
import type { Catalog, ControlTransport, Settings } from './devices.ts';

const settings: Settings = { version: 1, wakeTarget: null, sunshinePort: 47989 };
const catalog: Catalog = {
  checkedAt: 1700000000000,
  devices: [{ id: 'local_pc', name: 'Test PC', status: 'Online', detail: 'test',
    capabilities: [{ actionId: 'shutdown', enabled: true, reason: null, confirmationRequired: true }] }],
};
function transport(overrides: Partial<ControlTransport> = {}): ControlTransport {
  return {
    readCatalog: async () => catalog, readSettings: async () => settings,
    saveSettings: async () => {}, execute: async () => ({ message: 'ok' }), ...overrides,
  };
}
test('an unconfirmed shutdown never reaches the transport', async () => {
  let calls = 0;
  const center = new ControlCenter(transport({ execute: async () => { calls++; return { message: 'ok' }; } }));
  await center.refresh();
  assert.equal(await center.execute({ deviceId: 'local_pc', actionId: 'shutdown' }), false);
  assert.equal(calls, 0);
});
test('a wrong target cannot reuse another device capability', async () => {
  let calls = 0;
  const center = new ControlCenter(transport({ execute: async () => { calls++; return { message: 'ok' }; } }));
  await center.refresh();
  assert.equal(await center.execute({ deviceId: 'gamer_wake', actionId: 'shutdown', confirmed: true }), false);
  assert.equal(calls, 0);
});
test('failed verification removes the previous online snapshot', async () => {
  let fail = false;
  const center = new ControlCenter(transport({ readCatalog: async () => {
    if (fail) throw new Error('unavailable');
    return catalog;
  } }));
  await center.refresh();
  assert.equal(center.getSnapshot().catalog?.devices[0].status, 'Online');
  fail = true;
  await center.refresh();
  assert.equal(center.getSnapshot().catalog, null);
  assert.equal(center.getSnapshot().error, 'unavailable');
});
test('concurrent refreshes share the operation lock', async () => {
  let calls = 0;
  let release!: () => void;
  const waiting = new Promise<void>(resolve => { release = resolve; });
  const center = new ControlCenter(transport({ readCatalog: async () => { calls++; await waiting; return catalog; } }));
  const first = center.refresh();
  await center.refresh();
  assert.equal(calls, 1);
  release();
  await first;
  assert.equal(center.getSnapshot().busy, null);
});
test('a rejected settings save preserves the accepted settings', async () => {
  const center = new ControlCenter(transport({ saveSettings: async () => { throw new Error('invalid port'); } }));
  await center.refresh();
  assert.equal(await center.saveSettings({ ...settings, sunshinePort: 0 }), false);
  assert.equal(center.getSnapshot().settings?.sunshinePort, 47989);
});
test('dispatch failure is not reported as successful execution', async () => {
  const center = new ControlCenter(transport({ execute: async () => { throw new Error('denied'); } }));
  await center.refresh();
  assert.equal(await center.execute({ deviceId: 'local_pc', actionId: 'shutdown', confirmed: true }), false);
  assert.equal(center.getSnapshot().feedback, '');
  assert.equal(center.getSnapshot().error, 'denied');
});