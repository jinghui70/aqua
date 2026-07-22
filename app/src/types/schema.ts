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
  enum?: InlineEnum;
  comment?: string;
}

export interface IndexField {
  code: string;
  direction: "ASC" | "DESC";
}

export interface Index {
  name?: string;
  fields: IndexField[];
  unique: boolean;
}

export interface Table {
  /** 运行时唯一 id(页签/route 标识,不持久化;加载/新建时生成) */
  id: string;
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
  bizTypeData?: { fields: Array<{ name: string; type: "string" | "number"; description?: string; required?: boolean; default?: string | number }> };
}

export interface AutoGenStrategyDefine {
  code: string;
  name: string;
  /** 参数说明(有参数时作 placeholder)。无参数则 undefined */
  paramDesc?: string;
}

export interface Project {
  version: string;
  /** 项目中文名(可选,旧 schema 兼容) */
  name?: string;
  basePackage: string;
  bizTypes: BizTypeDefine[];
  /** 自定义自动生成策略(内置 default/now 不存项目) */
  autoGenStrategies?: AutoGenStrategyDefine[];
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

/** 数据库支持信息(清单 + hidden/installed 状态,供配置页与下拉)。 */
export interface DatabaseInfo {
  name: string;
  label: string;
  category: "native" | "jdbc";
  defaultPort: number;
  needsSchema: boolean;
  generateAs: string | null;
  driverClass: string | null;
  reverseSupported: boolean;
  builtinDriver: boolean;
  hidden: boolean;
  installed: boolean;
  installedJar: string | null;
}

/** 数据集条目:一张表的全部数据行(key 用字段 code)。 */
export interface DatasetEntry {
  table: string;
  data: Array<Record<string, unknown>>;
}
