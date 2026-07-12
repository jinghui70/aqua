// 数据源列表(Pinia,内存态。持久化到 .dbconfig.json 后续任务)。
import { defineStore } from "pinia";
import { ref } from "vue";
import type { DbConfig } from "@/types/schema";

/** 数据源 = DbConfig + 唯一名称。 */
export interface DataSource extends DbConfig {
  /** 数据源名称(dev/test/prod 等) */
  sourceName: string;
}

export const useDataSourceStore = defineStore("datasource", () => {
  const sources = ref<DataSource[]>([]);

  function add(ds: DataSource): string | null {
    if (sources.value.some((s) => s.sourceName === ds.sourceName)) {
      return `数据源 ${ds.sourceName} 已存在`;
    }
    sources.value.push(ds);
    return null;
  }

  function update(sourceName: string, ds: DataSource) {
    const idx = sources.value.findIndex((s) => s.sourceName === sourceName);
    if (idx >= 0) sources.value[idx] = ds;
  }

  function remove(sourceName: string) {
    const idx = sources.value.findIndex((s) => s.sourceName === sourceName);
    if (idx >= 0) sources.value.splice(idx, 1);
  }

  return { sources, add, update, remove };
});
