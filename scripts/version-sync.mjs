// 把根 package.json 的 version 同步到 tauri.conf.json + Cargo.toml [workspace.package]
// 由 bumpp execute 钩子调用:此时根 package.json 已是新版本,bumpp all 已同步 app/package.json
// 只替换 version 行,保留文件原格式(不用 JSON.parse/stringify 避免 diff 噪音)
import fs from 'node:fs'

const v = JSON.parse(fs.readFileSync('package.json', 'utf8')).version
console.log(`[version-sync] ${v} -> tauri.conf.json + Cargo.toml`)

// tauri.conf.json(决定 dmg/exe 产物文件名版本)
const tauriPath = 'src-tauri/tauri.conf.json'
const tauri = fs.readFileSync(tauriPath, 'utf8')
const newTauri = tauri.replace(/"version":\s*"[^"]*"/, `"version": "${v}"`)
if (newTauri === tauri) {
  console.error('[version-sync] tauri.conf.json 未找到 version,中止')
  process.exit(1)
}
fs.writeFileSync(tauriPath, newTauri)

// Cargo.toml [workspace.package] version(各 crate version.workspace=true 继承)
const cargoPath = 'Cargo.toml'
const cargo = fs.readFileSync(cargoPath, 'utf8')
const newCargo = cargo.replace(/^version = ".*"/m, `version = "${v}"`)
if (newCargo === cargo) {
  console.error('[version-sync] Cargo.toml 未找到行首 version,中止')
  process.exit(1)
}
fs.writeFileSync(cargoPath, newCargo)

console.log('[version-sync] done')
