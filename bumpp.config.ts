import { defineConfig } from 'bumpp'

// pnpm release: 交互选版本 -> 同步 version -> commit + tag vX.Y.Z -> push -> 触发 CI
//   all:       git add all(提交所有版本改动)
//   workspace: 同步 app/package.json
//   execute:   同步 tauri.conf.json + Cargo.toml [workspace.package](version-sync.mjs)
export default defineConfig({
  all: true,
  workspace: true,
  execute: 'node scripts/version-sync.mjs',
})
