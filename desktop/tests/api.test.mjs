import { readFile } from 'node:fs/promises';
import assert from 'node:assert/strict';
import { test } from 'node:test';
import ts from 'typescript';

const source = await readFile(new URL('../src/services/api.ts', import.meta.url), 'utf8');
const { outputText } = ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ES2020, module: ts.ModuleKind.ESNext },
});
const { arcadeApi } = await import(`data:text/javascript;base64,${Buffer.from(outputText).toString('base64')}`);

test('requests use the same-origin API proxy and preserve guess text', async (t) => {
  let received;
  t.mock.method(globalThis, 'fetch', async (url, options) => {
    received = { url, options };
    return new Response('{"result":"InvalidCharacters"}', { status: 200 });
  });
  await arcadeApi.submitBoggleWord('a"b\\c');
  assert.equal(received.url, '/api/boggle/submit');
  assert.equal(received.options.method, 'POST');
  assert.deepEqual(JSON.parse(received.options.body), { word: 'a"b\\c' });
});

test('structured HTTP errors retain the useful backend message', async (t) => {
  t.mock.method(console, 'error', () => {});
  t.mock.method(globalThis, 'fetch', async () => new Response('{"error":"Word not in dictionary"}', { status: 400 }));
  await assert.rejects(arcadeApi.submitWordleGuess('ZZZZZ'), { message: 'Word not in dictionary' });
});

test('non-JSON HTTP failures retain their status and response', async (t) => {
  t.mock.method(console, 'error', () => {});
  t.mock.method(globalThis, 'fetch', async () => new Response('Backend unavailable', { status: 502 }));
  await assert.rejects(arcadeApi.getStatus(), { message: 'HTTP error 502: Backend unavailable' });
});

test('legacy success-status error responses are still rejected', async (t) => {
  t.mock.method(console, 'error', () => {});
  t.mock.method(globalThis, 'fetch', async () => new Response('{"error":"Round ended"}'));
  await assert.rejects(arcadeApi.submitBoggleWord('CAT'), { message: 'Round ended' });
});

test('settings updates send only the changed fields', async (t) => {
  t.mock.method(globalThis, 'fetch', async (url, options) => {
    assert.equal(url, '/api/settings');
    assert.deepEqual(JSON.parse(options.body), { reduced_motion: true });
    return new Response('{"sound_enabled":true,"music_enabled":false,"reduced_motion":true}');
  });
  assert.equal((await arcadeApi.updateSettings({ reduced_motion: true })).reduced_motion, true);
});
