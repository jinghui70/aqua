// 命名转换:snake_case ↔ camelCase/PascalCase。对齐 Rust `generators/java/naming.rs`
// (保存文件名需与后端生成的类名/prop 一致,故规则须同源:按 _ 分割,每段首字母大写余小写)。

function capitalizeFirst(s: string): string {
  if (!s) return "";
  return s[0].toUpperCase() + s.slice(1).toLowerCase();
}

/** 蛇形 → 驼峰 (USER_NAME → userName)。 */
export function snakeToCamel(code: string): string {
  const parts = code.split("_").filter((p) => p.length > 0);
  if (parts.length === 0) return "";
  return parts[0].toLowerCase() + parts.slice(1).map(capitalizeFirst).join("");
}

/** 蛇形 → PascalCase (USER_INFO → UserInfo)。 */
export function snakeToPascal(code: string): string {
  return code
    .split("_")
    .filter((p) => p.length > 0)
    .map(capitalizeFirst)
    .join("");
}
