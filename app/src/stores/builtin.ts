// 内置业务类型(打包资源文件加载,只读)。与项目自定义 bizType 分离存储。
import { defineStore } from "pinia";
import { ref } from "vue";
import type { BizTypeDefine } from "@/types/schema";
import { useTauri } from "@/composables/useTauri";

export const useBuiltinStore = defineStore("builtin", () => {
  const tauri = useTauri();
  const bizTypes = ref<BizTypeDefine[]>([]);
  const loaded = ref(false);

  /** 加载内置清单(App 启动调一次,失败由 useTauri 统一提示)。 */
  async function load() {
    if (loaded.value) return;
    bizTypes.value = await tauri.builtinBiztypesLoad();
    loaded.value = true;
  }

  /** 判断 code 是否为内置(用于禁删改/重名校验)。 */
  function isBuiltin(code: string): boolean {
    return bizTypes.value.some((b) => b.bizType === code);
  }

  return { bizTypes, loaded, load, isBuiltin };
});
