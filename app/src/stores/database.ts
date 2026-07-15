// 数据库支持清单(应用级,缓存 listDatabases 结果)。
import { acceptHMRUpdate, defineStore } from "pinia";
import { ref, computed } from "vue";
import type { DatabaseInfo } from "@/types/schema";
import { useTauri } from "@/composables/useTauri";

export const useDatabaseStore = defineStore("database", () => {
  const tauri = useTauri();
  const databases = ref<DatabaseInfo[]>([]);
  const loaded = ref(false);

  async function load() {
    databases.value = await tauri.listDatabases();
    loaded.value = true;
  }

  /** 生成下拉:有驱动的(内置或已装)。 */
  const generatable = computed(() =>
    databases.value.filter((d) => d.builtinDriver || d.installed)
  );

  /** 反解下拉:有驱动且支持反解。 */
  const reversible = computed(() =>
    databases.value.filter((d) => (d.builtinDriver || d.installed) && d.reverseSupported)
  );

  return { databases, loaded, load, generatable, reversible };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useDatabaseStore, import.meta.hot));
}
