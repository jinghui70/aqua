// 字段剪贴板:跨表/跨项目复用字段(独立于 project,切换项目不清空)。
import { acceptHMRUpdate, defineStore } from "pinia";
import { computed, ref } from "vue";
import type { Field } from "@/types/schema";

export const useClipboardStore = defineStore("clipboard", () => {
  const fields = ref<Field[]>([]);

  const has = computed(() => fields.value.length > 0);

  // 深拷贝入/出,避免与源(选中字段)/目标(paste 后)共享引用
  function set(list: Field[]) {
    fields.value = JSON.parse(JSON.stringify(list));
  }
  function get(): Field[] {
    return JSON.parse(JSON.stringify(fields.value));
  }
  function clear() {
    fields.value = [];
  }

  return { fields, has, set, get, clear };
});

if (import.meta.hot) acceptHMRUpdate(useClipboardStore, import.meta.hot);
