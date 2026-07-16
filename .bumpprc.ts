import { defineConfig } from 'bumpp'

// pnpm release: 交互选版本 -> 同步 4 处 version -> commit + tag vX.Y.Z -> push -> 触发 CI
//   all:      同步根 + app 的 package.json
//   execute:  同步 tauri.conf.json + Cargo.toml(产物/二进制版本号)
export default defineConfig({
  all: true,
  execute: 'node scripts/version-sync.mjs',
})
