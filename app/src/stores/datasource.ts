// 数据源列表(Pinia)。持久化到项目对应配置文件,密码 AES 加密。
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
  /** 当前项目完整路径;空表示项目未保存,仅内存态。 */
  const projectPath = ref<string>("");
  /** 串行化落盘:保证写顺序与调用顺序一致,后写覆盖前写,最终态=最新内存态。 */
  let flushChain: Promise<void> = Promise.resolve();

  /** 从项目路径加载数据源(project open 时调)。无路径则清空。 */
  async function load(path: string) {
    projectPath.value = path;
    if (!path) {
      sources.value = [];
      return;
    }
    const wire = await tauri.datasourceLoad(path);
    sources.value = wire.map(fromWire);
  }

  /** 落盘到项目对应的配置文件。无项目路径(项目未保存)时跳过,仅保留内存态。
   *  多次调用串行排队,避免并发写文件互相覆盖。 */
  function persist(): Promise<void> {
    if (!projectPath.value) return Promise.resolve();
    // 每次排队时重新读取最新 sources,保证最后一次写反映最终内存态
    const run = () =>
      tauri.datasourceSave(
        projectPath.value,
        sources.value.map(toWire)
      );
    flushChain = flushChain.then(run, run);
    return flushChain;
  }

  /** 设置项目路径并把当前内存态数据源落盘(project 首次保存/另存时调)。 */
  async function bindDirAndPersist(path: string) {
    projectPath.value = path;
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
    projectPath,
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
