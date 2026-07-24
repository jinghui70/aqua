import { defineConfig, presetUno, presetIcons } from "unocss";
import { presetRemToPx } from "@unocss/preset-rem-to-px";

// baseFontSize: 4 -> 1 单位 = 1px(如 p-20 = 20px, gap-8 = 8px)
// 字号用 text-{n} 自定义规则,同样 1 单位 = 1px(如 text-14 = 14px)
export default defineConfig({
  presets: [
    presetUno(),
    presetRemToPx({ baseFontSize: 4 }),
    // 图标:i-mdi-* class 用 iconify mdi 集(构建期内联 SVG,无运行时依赖)。
    // extraProperties 让图标与文字基线对齐;尺寸靠 w-/h- 显式给(presetRemToPx 下 1em=4px 太小)。
    presetIcons({
      extraProperties: { display: "inline-block", "vertical-align": "middle" },
    }),
  ],
  rules: [
    [/^text-(\d+)$/, ([, n]) => ({ "font-size": `${n}px` })],
  ],
});
