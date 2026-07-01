import { spawnSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import { join } from 'node:path'

const env = { ...process.env }

if (process.platform === 'win32' && env.USERPROFILE) {
  const cargoBin = join(env.USERPROFILE, '.cargo', 'bin')
  if (existsSync(join(cargoBin, 'cargo.exe'))) {
    env.PATH = `${cargoBin};${env.PATH ?? ''}`
  }
}

// Windows Schannel can fail Cargo registry downloads when the certificate
// revocation server is unreachable. Scope this workaround to the desktop build.
if (process.platform === 'win32' && env.CARGO_HTTP_CHECK_REVOKE === undefined) {
  env.CARGO_HTTP_CHECK_REVOKE = 'false'
}

const result = spawnSync('bun', ['run', 'tauri', '--', 'build'], {
  stdio: 'inherit',
  env,
})

process.exit(result.status ?? 1)
