// Uses inherited stdio so compiler output stays visible and no Node named pipes are needed.
import { existsSync, mkdirSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
const root = fileURLToPath(new URL('../', import.meta.url));
const local = join(root, '.tools', 'cargo', 'bin', process.platform === 'win32' ? 'cargo.exe' : 'cargo');
const env = { ...process.env };
// Keep compiler/linker scratch files inside the project, separate from shared system TEMP policies.
const scratch = join(root, '.tools', 'tmp');
mkdirSync(scratch, { recursive: true });
env.TMP = scratch;
env.TEMP = scratch;
if (existsSync(local)) {
  env.CARGO_HOME = join(root, '.tools', 'cargo');
  env.RUSTUP_HOME = join(root, '.tools', 'rustup');
  env.PATH = join(env.CARGO_HOME, 'bin') + (process.platform === 'win32' ? ';' : ':') + env.PATH;
}
const result = spawnSync(existsSync(local) ? local : 'cargo', process.argv.slice(2), { cwd: root, env, stdio: 'inherit', windowsHide: true });
if (result.error) { console.error(result.error.message); process.exit(1); }
process.exit(result.status ?? 1);
