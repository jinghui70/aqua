// aqua schema 类型定义,对齐 Rust aqua_core::schema。
// 字段名驼峰,与 Rust #[serde(rename)] 后的 JSON 一致。

export enum DataType {
  Varchar = "VARCHAR",
  Clob = "CLOB",
  Tinyint = "TINYINT",
  Int = "INT",
  Long = "LONG",
  Decimal = "DECIMAL",
  Date = "DATE",
  Datetime = "DATETIME",
  Blob = "BLOB",
}

export interface AutoGenerate {
  enabled: boolean;
  strategy: string;
  param?: string;
  timing: "INSERT" | "INSERT_UPDATE";
}

export interface InlineEnumValue {
  id: string;
  name: string;
  code?: string;
  color?: string;
}

export interface InlineEnum {
  name: string;
  hasCode?: boolean;
  values: InlineEnumValue[];
}

export interface Field {
  prop: string;
  code: string;
  name: string;
  dataType: DataType;
  length?: number;
  precision?: number;
  scale?: number;
  bizType?: string;
  bizTypeData?: unknown;
  isKey?: boolean;
  notNull?: boolean;
  defaultValue?: string;
  autoGenerate?: AutoGenerate;
  enum?: string | InlineEnum;
  comment?: string;
}

export interface Index {
  name?: string;
  fields: string[];
  unique: boolean;
}

export interface Table {
  code: string;
  name: string;
  group: string;
  fields: Field[];
  indexes?: Index[];
  comment?: string;
}

export interface GroupDefine {
  code: string;
  name: string;
}

export interface BizTypeDefine {
  bizType: string;
  name: string;
  description?: string;
  supportedDataTypes: Array<{
    dataType: DataType;
    defaultLength?: number;
    defaultPrecision?: number;
    defaultScale?: number;
  }>;
  bizTypeData?: { fields: Array<{ name: string; type: "string" | "number"; description?: string; required?: boolean }> };
}

export interface EnumDefine {
  code: string;
  name: string;
  package: string;
  hasCode?: boolean;
  values: InlineEnumValue[];
}

export interface Project {
  version: string;
  basePackage: string;
  bizTypes: BizTypeDefine[];
  enums: EnumDefine[];
  groups: GroupDefine[];
  tables: Table[];
}

export interface ValidationError {
  path: string;
  message: string;
}

export interface DbConfig {
  dialect: string;
  host: string;
  port: number;
  user: string;
  password: string;
  database: string;
  schema?: string;
}

/** 数据集条目:一张表的全部数据行(key 用字段 code)。 */
export interface DatasetEntry {
  table: string;
  data: Array<Record<string, unknown>>;
}
