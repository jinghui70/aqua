#!/usr/bin/env node
// 构建 connector.jar 并复制到 src-tauri/resources/(打包 resource)。
// 跨平台:Node 内置 fs/child_process,无额外依赖。
// beforeBuildCommand 调用(全量重建),beforeDevCommand 调用 --if-missing(仅缺失时构建)。

import { execSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const connectorDir = join(root, "connector");
const srcJar = join(connectorDir, "target", "connector.jar");
const destDir = join(root, "src-tauri", "resources");
const destJar = join(destDir, "connector.jar");

// dev 模式(--if-missing):jar 已存在则跳过 Maven 构建,避免每次 tauri dev 跑 mvn。
// connector 源码改动后需手动 `pnpm build:connector` 全量重建。
const ifMissing = process.argv.includes("--if-missing");
if (ifMissing && existsSync(destJar)) {
  console.log(`[build:connector] 已存在,跳过: ${destJar}`);
  process.exit(0);
}

console.log("[build:connector] mvn package ...");
execSync("mvn -q package", { cwd: connectorDir, stdio: "inherit" });

if (!existsSync(srcJar)) {
  console.error(`[build:connector] 期望产物不存在: ${srcJar}`);
  process.exit(1);
}

mkdirSync(destDir, { recursive: true });
copyFileSync(srcJar, destJar);
console.log(`[build:connector] copied -> ${destJar}`);
