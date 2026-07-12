import { defineConfig, presetUno } from "unocss";
import { presetRemToPx } from "@unocss/preset-rem-to-px";

// baseFontSize: 4 -> 1 单位 = 1px(如 p-20 = 20px, gap-8 = 8px)
// 字号用 text-{n} 自定义规则,同样 1 单位 = 1px(如 text-14 = 14px)
export default defineConfig({
  presets: [
    presetUno(),
    presetRemToPx({ baseFontSize: 4 }),
  ],
  rules: [
    [/^text-(\d+)$/, ([, n]) => ({ "font-size": `${n}px` })],
  ],
});
