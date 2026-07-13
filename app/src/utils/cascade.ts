// 删除级联保护:按表聚合统计引用 + 统一提示格式(bizType 与 enum 共用)。
import type { Project } from "@/types/schema";

/** 引用统计:按表聚合,返回每张引用表的 code 与字段数。 */
export interface RelatedTable {
  table: string;
  count: number;
}

/** 遍历项目所有字段,predicate 命中的按表聚合。 */
export function collectRelatedTables(
  project: Project | null | undefined,
  predicate: (field: Project["tables"][number]["fields"][number]) => boolean
): RelatedTable[] {
  if (!project) return [];
  const res: RelatedTable[] = [];
  for (const t of project.tables) {
    let count = 0;
    for (const f of t.fields) if (predicate(f)) count++;
    if (count > 0) res.push({ table: t.code, count });
  }
  return res;
}

/** 生成删除确认提示 HTML(表级聚合)。kind 为"业务类型"或"枚举"。 */
export function buildCascadePrompt(
  kind: "业务类型" | "枚举",
  code: string,
  related: RelatedTable[]
): string {
  if (!related.length) return `确认删除${kind} ${code}?`;
  const total = related.reduce((s, r) => s + r.count, 0);
  const tables = related.map((r) => r.table);
  const shown = tables.slice(0, 8).join("、") + (tables.length > 8 ? " …" : "");
  const setting = kind === "业务类型" ? "业务类型" : "枚举";
  return [
    `${kind} <b>${code}</b> 被以下 <b>${tables.length}</b> 张表使用:`,
    shown,
    `共 <b>${total}</b> 个字段引用,删除将清除这些字段的${setting}设置。确认删除?`,
  ].join("<br/>");
}
