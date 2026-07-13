# 技术设计:内置业务类型加载

## 分层

- **aqua-core**:`BizTypeDataField` 加可选 `default` 字段(参数默认值)。`BizTypeDefine` 顶层不加 builtin 字段。
- **src-tauri**:新增 `builtin_biztypes_load` command,读资源文件返回 `Vec<BizTypeDefine>`。
- **前端**:新增 `stores/builtin.ts` 持有内置 bizTypes;BizTypeManage 与 FieldDetailDialog 合并展示;选 bizType 时用 default 初始化 bizTypeData。

## 参数默认值(结构扩展)

`BizTypeDataField` 加 `default`(可选,跟随 field type 为 string|number):

```rust
pub struct BizTypeDataField {
    pub name: String,
    #[serde(rename = "type")] pub field_type: BizTypeDataFieldType,
    #[serde(skip_serializing_if = "Option::is_none")] pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub required: Option<bool>,
    #[serde(rename = "default", default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
}
```
`default` 是 Rust 关键字,字段名用 `default_value` + serde rename。`serde_json::Value` 兼容 string/number。可选 + `default`,旧 schema 无该字段照常反序列化。

TS:`BizTypeDataField` 加 `default?: string | number;`

## 清单文件

`src-tauri/resources/builtin-biztypes.json`:
```json
{
  "bizTypes": [
    {
      "bizType": "Date",
      "name": "日期",
      "description": "格式化日期字符串",
      "supportedDataTypes": [
        { "dataType": "VARCHAR", "defaultLength": 8 }
      ],
      "bizTypeData": {
        "fields": [
          { "name": "format", "type": "string", "default": "YYYYMMDD", "required": true, "description": "日期格式" }
        ]
      }
    }
  ]
}
```

## tauri.conf.json

`bundle.resources` 加:
```json
"resources": ["resources/builtin-biztypes.json"]
```
dev 模式下 `resource_dir()` 解析到 `src-tauri/`,故文件放 `src-tauri/resources/` 可被 dev/prod 统一读取。

## src-tauri command

新建 `src-tauri/src/commands/builtin.rs`:

```rust
use aqua_core::schema::BizTypeDefine;
use serde::Deserialize;
use tauri::{AppHandle, Manager, Runtime};
use tauri::path::BaseDirectory;

#[derive(Deserialize)]
struct BuiltinFile { #[serde(rename = "bizTypes")] biz_types: Vec<BizTypeDefine> }

#[tauri::command]
pub async fn builtin_biztypes_load<R: Runtime>(app: AppHandle<R>) -> Result<Vec<BizTypeDefine>, String> {
    let path = app.path()
        .resolve("resources/builtin-biztypes.json", BaseDirectory::Resource)
        .map_err(|e| format!("定位内置清单失败: {}", e))?;
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取内置清单失败: {}", e))?;
    let file: BuiltinFile = serde_json::from_str(&content)
        .map_err(|e| format!("内置清单 JSON 非法: {}", e))?;
    Ok(file.biz_types)
}
```

注册进 `commands/mod.rs` + `lib.rs` invoke_handler。

## 前端

### `stores/builtin.ts`(新)

```ts
export const useBuiltinStore = defineStore("builtin", () => {
  const bizTypes = ref<BizTypeDefine[]>([]);
  const loaded = ref(false);
  async function load() {
    if (loaded.value) return;
    bizTypes.value = await useTauri().builtinBiztypesLoad();
    loaded.value = true;
  }
  return { bizTypes, load };
});
```

### App 启动加载

`App.vue` onMounted 调 `useBuiltinStore().load()`(失败已由 useTauri 统一弹错)。

### BizTypeManage 改造

- `bizTypes` computed 改为合并:`[...builtin.bizTypes(只读), ...project.bizTypes(可改)]`。
- 每条加 `builtin` 派生标记(基于是否在 builtin store)控制 UI:内置条目隐藏删按钮、禁用点击编辑。
- `addBizType` 重名校验:`[...builtin, ...project].some(b => b.bizType === code)`。
- `removeBizType` 仅对自定义生效;内置条目不会进入此路径(UI 已禁)。
- 编辑表单:选中内置条目时表单只读或提示"内置业务类型不可编辑"(选只读更直观)。

### FieldDetailDialog 改造

- `bizTypes` computed 改为合并内置 + 自定义。
- `availableBizTypes`(类型联动过滤)对合并后的列表生效,内置条目同样参与过滤。
- **默认值初始化**:新增 `initBizTypeData(def)`:
  ```ts
  function initBizTypeData(def: BizTypeDefine): unknown {
    const fields = def.bizTypeData?.fields ?? [];
    if (!fields.length) return undefined;
    if (fields.length === 1) return fields[0].default;
    const obj: Record<string, unknown> = {};
    for (const f of fields) if (f.default !== undefined) obj[f.name] = f.default;
    return obj;
  }
  ```
  `onBizTypeChange` 选普通 bizType 后调 `draft.bizTypeData = initBizTypeData(def)`(顺带修复切 bizType 不清旧 bizTypeData 的缺陷)。其余联动逻辑不变。

## 风险

- Tauri 2.x `BaseDirectory::Resource` 在 dev 模式的解析路径需实测确认。若 dev 读不到,fallback 用 `app.path().resource_dir()` 手动 join。
- 内置与自定义同 code 冲突:前端新建校验拦截;若用户手工编辑 schema.json 造出冲突,展示去重(内置优先),不报错。
