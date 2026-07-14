// 数据源列表(Pinia)。持久化到项目目录 .dbconfig.json,密码 AES 加密。
import { acceptHMRUpdate, defineStore } from "pinia";
import { ref } from "vue";
import type { DbConfig } from "@/types/schema";
import { useTauri } from "@/composables/useTauri";

/** 数据源 = DbConfig + 唯一名称。 */
export interface DataSource extends DbConfig {
  /** 数据源名称(dev/test/prod 等) */
  sourceName: string;
}

/** 拆分 sourceName,得到后端 wire 结构 [name, DbConfig]。 */
function toWire(ds: DataSource): [string, DbConfig] {
  const { sourceName, ...cfg } = ds;
  return [sourceName, cfg];
}

/** 后端 wire 结构 [name, DbConfig] → DataSource。 */
function fromWire([sourceName, cfg]: [string, DbConfig]): DataSource {
  return { sourceName, ...cfg };
}

export const useDataSourceStore = defineStore("datasource", () => {
  const tauri = useTauri();
  const sources = ref<DataSource[]>([]);
  /** 当前项目目录;空表示项目未保存,仅内存态。 */
  const projectDir = ref<string>("");
  /** 串行化落盘:保证写顺序与调用顺序一致,后写覆盖前写,最终态=最新内存态。 */
  let flushChain: Promise<void> = Promise.resolve();

  /** 从项目目录加载数据源(project open 时调)。无目录则清空。 */
  async function load(dir: string) {
    projectDir.value = dir;
    if (!dir) {
      sources.value = [];
      return;
    }
    const wire = await tauri.datasourceLoad(dir);
    sources.value = wire.map(fromWire);
  }

  /** 落盘到项目目录。无目录(项目未保存)时跳过,仅保留内存态。
   *  多次调用串行排队,避免并发写文件互相覆盖。 */
  function persist(): Promise<void> {
    if (!projectDir.value) return Promise.resolve();
    // 每次排队时重新读取最新 sources,保证最后一次写反映最终内存态
    const run = () =>
      tauri.datasourceSave(
        projectDir.value,
        sources.value.map(toWire)
      );
    flushChain = flushChain.then(run, run);
    return flushChain;
  }

  /** 设置项目目录并把当前内存态数据源落盘(project 首次保存/另存时调)。 */
  async function bindDirAndPersist(dir: string) {
    projectDir.value = dir;
    await persist();
  }

  function add(ds: DataSource): string | null {
    if (sources.value.some((s) => s.sourceName === ds.sourceName)) {
      return `数据源 ${ds.sourceName} 已存在`;
    }
    sources.value.push(ds);
    void persist();
    return null;
  }

  function update(sourceName: string, ds: DataSource) {
    const idx = sources.value.findIndex((s) => s.sourceName === sourceName);
    if (idx >= 0) {
      sources.value[idx] = ds;
      void persist();
    }
  }

  function remove(sourceName: string) {
    const idx = sources.value.findIndex((s) => s.sourceName === sourceName);
    if (idx >= 0) {
      sources.value.splice(idx, 1);
      void persist();
    }
  }

  return {
    sources,
    projectDir,
    load,
    persist,
    bindDirAndPersist,
    add,
    update,
    remove,
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useDataSourceStore, import.meta.hot));
}
