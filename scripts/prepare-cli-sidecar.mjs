import { copyFileSync, mkdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { execFileSync } from 'node:child_process'

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)))
const tauriDir = join(rootDir, 'src-tauri')
const platformNames = {
    win32: 'windows',
    darwin: 'macos',
}
const architectureNames = {
    arm64: 'aarch64',
    ia32: 'x86',
    x64: 'x86_64',
}
const platform = platformNames[process.env.TAURI_ENV_PLATFORM ?? process.platform]
    ?? process.env.TAURI_ENV_PLATFORM
    ?? process.platform
const architecture = architectureNames[process.env.TAURI_ENV_ARCH ?? process.arch]
    ?? process.env.TAURI_ENV_ARCH
    ?? process.arch

const triples = {
    windows: {
        x86_64: 'x86_64-pc-windows-msvc',
        aarch64: 'aarch64-pc-windows-msvc',
        x86: 'i686-pc-windows-msvc',
    },
    macos: {
        aarch64: 'aarch64-apple-darwin',
        x86_64: 'x86_64-apple-darwin',
    },
    linux: {
        x86_64: 'x86_64-unknown-linux-gnu',
        aarch64: 'aarch64-unknown-linux-gnu',
    },
}

const target = triples[platform]?.[architecture]

if (!target) {
    throw new Error(`Unsupported Tauri target: ${platform}/${architecture}`)
}

const executable = platform === 'windows' ? 'adConverter-cli.exe' : 'adConverter-cli'
const source = join(tauriDir, 'target', target, 'release', executable)
const destination = join(tauriDir, 'binaries', `adConverter-cli-${target}${platform === 'windows' ? '.exe' : ''}`)

execFileSync(
    'cargo',
    ['build', '--manifest-path', 'Cargo.toml', '--release', '--bin', 'adConverter-cli', '--target', target],
    {
        cwd: tauriDir,
        env: {
            ...process.env,
            // The CLI builds before its own sidecar exists.
            TAURI_CONFIG: JSON.stringify({ bundle: { externalBin: null } }),
        },
        stdio: 'inherit',
    },
)

mkdirSync(dirname(destination), { recursive: true })
copyFileSync(source, destination)
