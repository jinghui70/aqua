# 表编辑页改进 - 技术设计

## 架构边界

- **前端**(运行时态):Table 加 `id`(页签标识),IndexField 带方向。负责 id 生成、UI、页签/route。
- **后端**(持久态):`Table` struct **不加** id(serde 无 `deny_unknown_fields`,前端发送的 id 字段反序列化时被忽略,文件不持久化 id)。`Index.fields` 改 `Vec<IndexField>` 含方向。负责 DDL 生成、diff。
- 解耦原则:code = 可改的展示属性;id = 稳定的运行时标识(页签/route 用)。

## 数据结构变更

### 前端 `app/src/types/schema.ts`
```ts
export interface IndexField {
  code: string;
  direction: "ASC" | "DESC";
}
export interface Index {
  name?: string;
  fields: IndexField[];   // 原 string[]
  unique: boolean;
}
export interface Table {
  id: string;             // 新增,运行时,不持久化(后端 struct 无此字段)
  code: string;
  // ...其余不变
}
```
- `Table.id` 必填;加载项目时 map 补 `crypto.randomUUID()`(所有加载路径必须补全)。

### 后端 `crates/aqua-core/src/schema/table.rs`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Direction { Asc, Desc }   // 序列化为 "ASC"/"DESC"

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndexField {
    pub code: String,
    pub direction: Direction,
}
// 反序列化兼容旧格式:纯字符串(默认 ASC)或 {code,direction}
impl<'de> Deserialize<'de> for IndexField { /* untagged: Str(code) | Obj{code,direction} */ }

pub struct Index {
    pub name: Option<String>,
    pub fields: Vec<IndexField>,   // 原 Vec<String>
    pub unique: bool,
}
// Table 不变(不加 id)
```

## 数据流

### R1 页签 id 解耦
- `openTable`(`project.ts:169`):`key: table:${table.id}`、`title: table.code`、`path: /table/${table.id}`。title 仍用 code(展示),key/path 用 id(稳定)。
- `router/index.ts`:`/table/:code` -> `/table/:id`,`props: true`。
- `TableEditor.vue:17`:`tables.find(t => t.id === props.id)`(原 find by code)。
- `openTab`:按 key(id)去重(不变)。
- 加载项目(`openProject`):后端返回 tables(无 id),前端 `.map(t => ({ ...t, id: crypto.randomUUID() }))`。
- 新建表(`addTable`):生成 id 后 push。
- 表 code 编辑:store 加 `updateTable(id, code, name)`--用 id 定位表,改 code(唯一校验,除自己)+ name。无需级联(id 不变,页签不失效)。`GroupTreeAside.vue` 表对话框 code 去 `disabled`,`tableEditingCode` -> `tableEditingId`,`confirmTableDialog` edit 分支调 `updateTable`。
- 树 node-key(`GroupTreeAside.vue:32,39`):表节点改用 `table:${t.id}`(原 `table:${t.code}`),改 code 时树节点 key 不变、不重建。分组节点保持 `group:${g.code}`(分组无运行时 id;改分组 code 时 key 变,el-tree 重渲染,分组少影响小)。

### R4 索引 DDL
- `generate_index`(`ddl/index.rs:6`):`fields` 映射为 `CODE DIRECTION` join,即 `CREATE [UNIQUE] INDEX name ON TABLE (F1 ASC, F2 DESC);`。
- `auto_index_name`(`ddl/index.rs:30`):`fields` 用 `IndexField.code` join(方向不入名)。
- `diff_indexes`(`diff/mod.rs:152`):`index_key` 含方向(方向不同 = 索引变更 -> DROP+CREATE)。
- `alter.rs`:复用 `generate_index`(已含方向)。

### R3 弹窗
- `FieldDetailDialog.vue:242`:`width="600px"` -> `"900px"`,基本区表单改多列布局(利用宽度,code/prop、名称/类型等两列排布)。
- 备注(`:287-289`)从基本区移到弹窗最末(业务类型区 `:324-413` 之后)。

### R4 前端 IndexTab UI
- 现状:`el-select multiple`,`fields: string[]`。
- 改:索引字段改成行列表--每行 = 字段 select(`field.code` 作 option)+ 方向 select(ASC/DESC)+ 拖拽柄/删除。可增删行、拖拽排序。Sortable.js(复用 FieldsTab 模式)或上下移按钮。

## 兼容性迁移

- **旧索引数据**:`fields: ["a","b"]` -> 反序列化时 `IndexField` 自定义 Deserialize 接受纯字符串(默认 ASC)。旧项目文件无需转换,打开即兼容。新版本保存写出 `{code,direction}` 格式。
- **Table id**:纯运行时,文件无 id 字段。旧项目加载时补 id,新版本保存仍无 id。跨版本无障碍。
- **回滚风险**:新版本保存的索引文件(`{code,direction}` 格式)旧版本读不了(旧 `Vec<String>` 反序列化对象失败)。项目早期数据少,可接受;若需回滚,手动改 JSON 或从 git 旧版恢复。

## Tradeoffs

| 决策 | 取舍 |
|---|---|
| Table id 不持久化 | schema 文件纯净 ✅ / 刷新页面 id 丢失页签失效(桌面不刷新,接受)❌ |
| IndexMeta 保留 `Vec<String>`(不带方向) | R4 聚焦 schema+generators+前端 ✅ / 数据库导入的索引方向丢失(默认 ASC,import 后置)❌ |
| 索引字段引用 field code(不修级联 bug) | scope 控制 ✅ / 改 field code 索引引用失效(现存 bug,保持现状)❌ |

## 验证命令
- 前端:`cd app && pnpm exec vue-tsc --noEmit`
- 后端:`cargo check -p aqua-core`
