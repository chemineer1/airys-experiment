#!/usr/bin/env node
// Exercise the actual inline loader with small streamed responses. No DOM or
// browser packages are needed; browser/GPU startup is checked separately.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { setImmediate } from 'node:timers/promises';
import test from 'node:test';

const html = await readFile(new URL('../web/index.html', import.meta.url), 'utf8');
const source = html.match(/<script type="module">([\s\S]*?)<\/script>/)[1]
  .replace('__WASM_BYTES__', '10')
  .replace("import('./airy.js?v=__BUILD_VERSION__')", 'loadGlue()');
const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
const execute = new AsyncFunction(
  'document', 'window', 'fetch', 'loadGlue', 'WebGL2RenderingContext',
  'WebAssembly', 'console', 'location', source,
);

function response(chunks = [2, 3, 5], options = {}) {
  return new Response(new ReadableStream({
    pull(controller) {
      if (chunks.length) controller.enqueue(new Uint8Array(chunks.shift()));
      else controller.close();
    },
  }), {
    // Fetch already decompressed the stream: 2 transfer bytes, 10 decoded bytes.
    headers: { 'Content-Length': '2', 'Content-Encoding': 'gzip' },
    ...options,
  });
}

async function load({ fetchImpl = async () => response(), initImpl, wasm = WebAssembly } = {}) {
  const elements = new Map();
  const values = [];
  const messages = [];
  let reloads = 0;
  let downloadSignal;
  for (const id of ['status', 'status-message', 'progress', 'retry', 'simulation']) {
    const element = {
      hidden: id === 'retry',
      handlers: {},
      attributes: {},
      addEventListener(name, handler) { this.handlers[name] = handler; },
      setAttribute(name, value) { this.attributes[name] = value; },
      focus() {},
    };
    if (id === 'progress') Object.defineProperty(element, 'value', {
      set(value) { values.push(value); },
    });
    if (id === 'status-message') Object.defineProperty(element, 'textContent', {
      set(value) { messages.push(value); },
      get() { return messages.at(-1); },
    });
    elements.set(`#${id}`, element);
  }
  const loader = async () => ({ default: initImpl ?? (async ({ module_or_path }) => {
    assert.equal(module_or_path.headers.get('Content-Type'), 'application/wasm');
    await module_or_path.arrayBuffer();
    assert.match(messages.at(-1), /Starting/);
  }) });
  await execute(
    { querySelector: selector => elements.get(selector) },
    { addEventListener() {} },
    async (url, options) => {
      downloadSignal = options.signal;
      return fetchImpl(url, options);
    },
    loader, function WebGL2() {}, wasm, { error() {} },
    { reload() { reloads += 1; } },
  );
  return {
    status: elements.get('#status'),
    retry: elements.get('#retry'),
    canvas: elements.get('#simulation'),
    progress: elements.get('#progress'),
    values, messages, downloadSignal,
    get reloads() { return reloads; },
  };
}

function assertFailure(result, message) {
  assert.equal(result.status.hidden, false);
  assert.equal(result.progress.hidden, true);
  assert.equal(result.retry.hidden, false);
  assert.match(result.messages.at(-1), message);
  if (result.downloadSignal) assert.equal(result.downloadSignal.aborted, true);
}

test('decoded byte count drives progress despite compressed Content-Length', async () => {
  const result = await load();
  assert.deepEqual(result.values, [20, 50, 100]);
  assert.equal(result.status.hidden, true);
  assert.equal(result.retry.hidden, true);
  assert.equal(result.canvas.attributes['aria-busy'], 'false');
});

test('HTTP and network failures offer a working retry', async () => {
  for (const [fetchImpl, message] of [
    [async () => response([], { status: 503 }), /HTTP 503/],
    [async () => { throw new TypeError('Connection lost'); }, /connection/],
  ]) {
    const result = await load({ fetchImpl });
    assertFailure(result, message);
    result.retry.handlers.click();
    assert.equal(result.reloads, 1);
  }
});

test('incomplete or stale payloads never start the simulation', async () => {
  for (const [chunks, message] of [[[9], /incomplete/], [[11], /out of date/]]) {
    assertFailure(await load({ fetchImpl: async () => response(chunks) }), message);
  }
});

test('late buffered chunks cannot overwrite an earlier compilation error', async () => {
  let incoming;
  const result = await load({
    fetchImpl: async () => new Response(new ReadableStream({ start(controller) { incoming = controller; } })),
    initImpl: async () => { throw new WebAssembly.CompileError('Invalid module'); },
  });
  assertFailure(result, /could not start/);
  const failure = result.messages.at(-1);
  incoming.enqueue(new Uint8Array(10));
  incoming.close();
  await setImmediate();
  assert.equal(result.messages.at(-1), failure);
  assert.deepEqual(result.values, []);
});

test('compiler cancellation does not masquerade as an incomplete download', async () => {
  assertFailure(await load({
    fetchImpl: async () => new Response(new ReadableStream()),
    initImpl: async ({ module_or_path }) => {
      await setImmediate(); // Let the stream begin a pending read.
      await module_or_path.body.cancel('Invalid module');
      throw new WebAssembly.CompileError('Invalid module');
    },
  }), /could not start/);
});

test('unsupported browsers and lost graphics contexts get specific errors', async () => {
  assertFailure(await load({ wasm: null }), /needs WebAssembly/);
  const running = await load();
  running.canvas.handlers.webglcontextlost();
  assertFailure(running, /Graphics context lost/);
});
