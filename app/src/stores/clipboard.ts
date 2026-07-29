// 字段/表剪贴板:跨表/跨项目复用(独立于 project,切换项目不清空)。
import { acceptHMRUpdate, defineStore } from "pinia";
import { computed, ref } from "vue";
import type { Field, Table } from "@/types/schema";

export const useClipboardStore = defineStore("clipboard", () => {
  const fields = ref<Field[]>([]);
  const tables = ref<Table[]>([]);

  const has = computed(() => fields.value.length > 0);
  const hasTables = computed(() => tables.value.length > 0);

  // 深拷贝入/出,避免与源(选中)/目标(paste 后)共享引用
  function set(list: Field[]) {
    fields.value = JSON.parse(JSON.stringify(list));
  }
  function get(): Field[] {
    return JSON.parse(JSON.stringify(fields.value));
  }
  function setTables(list: Table[]) {
    tables.value = JSON.parse(JSON.stringify(list));
  }
  function getTables(): Table[] {
    return JSON.parse(JSON.stringify(tables.value));
  }
  function clear() {
    fields.value = [];
  }

  return { fields, has, set, get, clear, tables, hasTables, setTables, getTables };
});

if (import.meta.hot) acceptHMRUpdate(useClipboardStore, import.meta.hot);
