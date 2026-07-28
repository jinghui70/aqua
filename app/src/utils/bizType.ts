// bizType↔dataType 联动纯函数: BizTypeEditDialog 与 FieldsTab 共用。
import type { BizTypeDefine, DataType, Field } from "@/types/schema";

export function bizTypeSupports(def: BizTypeDefine, dt: DataType): boolean {
  return def.supportedDataTypes.some((s) => s.dataType === dt);
}

// 填该 bizType 对指定 dataType 定义的默认 length/precision/scale(§3.4)。
export function applyDefaults(target: Field, def: BizTypeDefine, dt: DataType): void {
  const s = def.supportedDataTypes.find((x) => x.dataType === dt);
  if (!s) return;
  if (s.defaultLength != null) target.length = s.defaultLength;
  if (s.defaultPrecision != null) target.precision = s.defaultPrecision;
  if (s.defaultScale != null) target.scale = s.defaultScale;
}
