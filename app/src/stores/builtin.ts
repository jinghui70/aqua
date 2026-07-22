// 内置业务类型(打包资源文件加载,只读)+ 内置自动生成策略(硬编码)。
import { acceptHMRUpdate, defineStore } from "pinia";
import { ref } from "vue";
import type { BizTypeDefine, AutoGenStrategyDefine } from "@/types/schema";
import { useTauri } from "@/composables/useTauri";

/** 内置自动生成策略(写死代码,非外置 JSON) */
const BUILTIN_AUTO_GEN_STRATEGIES: AutoGenStrategyDefine[] = [
  { code: "default", name: "雪花id", paramDesc: "前缀" },
  { code: "now", name: "当前时间" },
];

export const useBuiltinStore = defineStore("builtin", () => {
  const tauri = useTauri();
  const bizTypes = ref<BizTypeDefine[]>([]);
  const autoGenStrategies = ref<AutoGenStrategyDefine[]>(BUILTIN_AUTO_GEN_STRATEGIES);
  const loaded = ref(false);

  async function load() {
    if (loaded.value) return;
    bizTypes.value = await tauri.builtinBiztypesLoad();
    loaded.value = true;
  }

  function isBuiltinBizType(code: string): boolean {
    return bizTypes.value.some((b) => b.bizType === code);
  }

  function isBuiltinStrategy(code: string): boolean {
    return autoGenStrategies.value.some((s) => s.code === code);
  }

  return { bizTypes, autoGenStrategies, loaded, load, isBuiltinBizType, isBuiltinStrategy };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useBuiltinStore, import.meta.hot));
}
