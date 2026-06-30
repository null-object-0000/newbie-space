import { spawnSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import { join } from 'node:path'

function windowsCargoPath() {
  if (process.platform !== 'win32' || !process.env.USERPROFILE) {
    return null
  }

  const cargoPath = join(process.env.USERPROFILE, '.cargo', 'bin', 'cargo.exe')
  return existsSync(cargoPath) ? cargoPath : null
}

function hasCommand(command, args = ['--version']) {
  if (!command) {
    return false
  }

  const result = spawnSync(command, args, {
    stdio: 'ignore',
  })
  return result.status === 0
}

const missing = []

if (!hasCommand('cargo') && !hasCommand(windowsCargoPath())) {
  missing.push('cargo (Rust toolchain)')
}

if (missing.length > 0) {
  console.error('\nDesktop build prerequisites are missing:')
  for (const item of missing) {
    console.error(`- ${item}`)
  }
  console.error('\nInstall Rust from https://rustup.rs/, then reopen the terminal and run `bun run build` again.\n')
  process.exit(1)
}
