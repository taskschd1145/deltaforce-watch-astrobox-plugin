// Precise CSS viewport screenshots from the same Rust-generated UI tree.
// Uses a separate headless browser profile and the DevTools protocol, not the user's browser.
import { spawn } from 'node:child_process';
import { readFile, writeFile, mkdir, readdir } from 'node:fs/promises';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { join } from 'node:path';
const root = fileURLToPath(new URL('../', import.meta.url));
const directory = join(root, 'docs', 'validation', 'previews');
const profile = join(root, '.tools', `capture-browser-${Date.now()}`);
await mkdir(profile, { recursive: true });
const browser = spawn('C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe', ['--headless=new', '--disable-gpu', '--no-first-run', '--no-default-browser-check', '--hide-scrollbars', '--remote-debugging-port=0', `--user-data-dir=${profile}`, 'about:blank'], { stdio: 'ignore', windowsHide: true });
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
let socket;
try {
  let endpoint;
  for (let attempt = 0; attempt < 100; attempt++) {
    try { const [port, path] = (await readFile(join(profile, 'DevToolsActivePort'), 'utf8')).trim().split(/\r?\n/); endpoint = `ws://127.0.0.1:${port}${path}`; break; } catch {}
    await delay(100);
  }
  if (!endpoint) throw new Error('Headless browser startup timed out.');
  socket = new WebSocket(endpoint);
  await new Promise((resolve, reject) => { socket.addEventListener('open', resolve, { once: true }); socket.addEventListener('error', reject, { once: true }); });
  let seq = 0;
  const requests = new Map();
  socket.addEventListener('message', event => { const data = JSON.parse(event.data); if (data.id && requests.has(data.id)) { const [resolve, reject] = requests.get(data.id); requests.delete(data.id); data.error ? reject(new Error(JSON.stringify(data.error))) : resolve(data.result); } });
  const call = (method, params = {}, sessionId) => new Promise((resolve, reject) => { const id = ++seq; requests.set(id, [resolve, reject]); socket.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) })); });
  const results = [];
  for (const name of (await readdir(directory)).filter(n => n.endsWith('.json')).sort()) {
    const payload = JSON.parse(await readFile(join(directory, name), 'utf8'));
    const { targetId } = await call('Target.createTarget', { url: 'about:blank' });
    const { sessionId } = await call('Target.attachToTarget', { targetId, flatten: true });
    await call('Page.enable', {}, sessionId);
    await call('Emulation.setDeviceMetricsOverride', { width: payload.width, height: payload.height, deviceScaleFactor: 1, mobile: false }, sessionId);
    await call('Page.navigate', { url: pathToFileURL(join(directory, name.replace('.json', '.html'))).href }, sessionId);
    await delay(100);
    const evaluated = await call('Runtime.evaluate', { expression: 'JSON.stringify({width:innerWidth,height:innerHeight,scrollWidth:document.documentElement.scrollWidth})', returnByValue: true }, sessionId);
    const metrics = JSON.parse(evaluated.result.value);
    if (metrics.width !== payload.width || metrics.height !== payload.height || metrics.scrollWidth > payload.width) throw new Error(`Viewport/overflow check failed for ${name}: ${JSON.stringify(metrics)}`);
    const shot = await call('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false }, sessionId);
    await writeFile(join(directory, name.replace('.json', '.png')), Buffer.from(shot.data, 'base64'));
    results.push({ name: name.replace('.json', ''), ...metrics, status: 'passed' });
    await call('Target.closeTarget', { targetId });
  }
  await writeFile(join(root, 'docs', 'validation', 'viewports.json'), JSON.stringify(results, null, 2) + '\n');
  console.log(JSON.stringify({ previews: results.length, exactViewports: true, horizontalOverflow: false }));
  await call('Browser.close');
} finally {
  socket?.close();
  if (!browser.killed) browser.kill();
}
