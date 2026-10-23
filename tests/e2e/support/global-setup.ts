import { execFileSync } from 'node:child_process';
import { REPO } from './server';

// The server binary with the UI engine and the demo show (built from its sources when missing
// or stale, decisions §40 15). Cargo only rebuilds what changed.
export default function globalSetup() {
  execFileSync('cargo', ['build', '-p', 'lumacore-station', '--features', 'http', '--bin', 'lumacore-server', '--bin', 'gen-demo'], {
    cwd: REPO,
    stdio: 'inherit',
  });
  execFileSync(`${REPO}/target/debug/gen-demo`, ['--if-stale'], { cwd: REPO, stdio: 'inherit' });
}
