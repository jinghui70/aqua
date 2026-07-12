// 最近项目(localStorage 持久化)。

const KEY = "aqua.recentProjects";
const MAX = 10;

export interface RecentProject {
  path: string;
  /** 最后打开时间戳(ms) */
  openedAt: number;
}

export function useRecentProjects() {
  function load(): RecentProject[] {
    try {
      const raw = localStorage.getItem(KEY);
      if (!raw) return [];
      return JSON.parse(raw) as RecentProject[];
    } catch {
      return [];
    }
  }

  function save(list: RecentProject[]) {
    localStorage.setItem(KEY, JSON.stringify(list.slice(0, MAX)));
  }

  /** 记录一次打开(去重 + 置顶 + 更新时间)。 */
  function record(path: string) {
    const list = load().filter((r) => r.path !== path);
    list.unshift({ path, openedAt: Date.now() });
    save(list);
  }

  /** 移除(文件不存在时)。 */
  function remove(path: string) {
    save(load().filter((r) => r.path !== path));
  }

  return { load, record, remove };
}
