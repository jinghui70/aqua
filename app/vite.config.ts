import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import UnoCSS from "unocss/vite";
import { fileURLToPath, URL } from "node:url";
import fs from "node:fs";

// 产品版本号(读 package.json,release 时 version-sync 同步),构建期注入供前端使用
const pkg = JSON.parse(fs.readFileSync(new URL("./package.json", import.meta.url), "utf-8"));

// Tauri 期望前端 dev server 在 1420 端口
export default defineConfig({
  plugins: [vue(), UnoCSS()],
  clearScreen: false,
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: "esnext",
    outDir: "dist",
  },
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
});
