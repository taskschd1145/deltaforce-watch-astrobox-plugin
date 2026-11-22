// Development bootstrap only; downloads the official rustup installer into the ignored project tools directory.
import { mkdir, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
const target = new URL('../.tools/rustup-init.exe', import.meta.url);
const url = 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe';
await mkdir(new URL('../.tools/', import.meta.url), { recursive: true });
const response = await fetch(url, { signal: AbortSignal.timeout(180000) });
if (!response.ok) throw new Error(`Official rustup download failed: HTTP ${response.status}`);
const bytes = Buffer.from(await response.arrayBuffer());
if (bytes.length < 1000000 || bytes.length > 40000000) throw new Error('Unexpected installer size.');
await writeFile(target, bytes);
console.log(JSON.stringify({ url, file: fileURLToPath(target), bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') }));
