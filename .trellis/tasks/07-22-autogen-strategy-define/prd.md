# 自动生成策略全局定义(类似业务类型)

## Goal

将自动生成策略从字段级硬编码(strategy 字符串 "default"/"now")提升为**全局定义**(内置 + 自定义),类似业务类型(BizTypeDefine)。字段引用策略 code,管理页统一维护。让策略可扩展(自定义策略),不再硬编码。

## 背景

当前 `field.autoGenerate = { enabled, strategy: String, param, timing }`,strategy 是字符串("default" 雪花 / "now" 当前时间),硬编码在 FieldDetail 下拉。无全局定义,无法自定义策略。

业务类型已有全局定义模式(BizTypeDefine:内置 + 自定义,配置中心 BizTypePanel 管理,字段引用 code)。自动生成策略对齐此模式。

## 需求

### 1. 全局策略定义 AutoGenStrategyDefine
- 结构:`code` / `name` / `paramDesc: Option<String>`(参数说明,有参数时用于字段编辑 placeholder)。
- 内置策略(写死代码,非外置 JSON):
  - `default`(雪花id),无参数(paramDesc None)
  - `now`(当前时间),有参数(paramDesc "yyyy-MM-dd HH:mm:ss 格式")
- 自定义策略:项目 `Project.autoGenStrategies: Vec<AutoGenStrategyDefine>`。
- 策略最多 1 个参数(字符串),有参数则 paramDesc 描述,无参数则 paramDesc None。

### 2. 字段引用
- `field.autoGenerate.strategy` 引用策略 code。
- `field.autoGenerate.param: Option<String>` 字段级(保留,仅当策略有参数时显示)。
- `field.autoGenerate.timing` 字段级(保留)。
- FieldDetail 策略下拉显示全局策略(name),值 = code。
- 策略无参数 -> 不显示 param 输入框。有参数 -> 显示,paramDesc 作 placeholder。

### 3. 管理页 AutoGenStrategyPanel
- 配置中心加"自动生成策略"导航 + AutoGenStrategyPanel(参考 BizTypePanel)。
- 左列表(内置 + 自定义)+ 右编辑(code 只读 tag / name 可编辑 / paramDesc textarea)。
- 定义策略时配置"是否有参数"(有则填 paramDesc,无则 paramDesc None)。
- 只读(全局 + 预置)用 readonly。
- 新建弹窗录 code + name。

### 4. 删除策略级联
- 删除自定义策略时,collectRelatedTables 找引用字段(autoGenerate.strategy === code),提示 + 级联清除(字段 autoGenerate = undefined)。

### 5. Java 生成
- @GeneratedValue(strategy = "code") -- strategy 引用全局 code(不变,但 code 来源全局)。

## 验收标准

- [ ] schema:AutoGenStrategyDefine(code/name)+ Project.autoGenStrategies + 内置(default/now)
- [ ] 配置中心 AutoGenStrategyPanel:列表(内置+自定义)+ 编辑(code tag/name)+ 新建弹窗 + 删除级联
- [ ] FieldDetail 策略下拉显示全局策略(name),值 code
- [ ] 字段 autoGenerate.strategy 引用全局 code;param/timing 字段级不变
- [ ] 删除策略级联清除引用字段 autoGenerate
- [ ] Java 生成 @GeneratedValue(strategy=code) 不回归
- [ ] cargo test/clippy + vue-tsc 全过,现有功能不回归

## 非目标

- 策略多参数(最多 1 个字符串参数)
- timing 策略级(保留字段级)
- 策略实现逻辑(策略只是 code/name/paramDesc 注册,生成逻辑仍在 Java 生成器按 code 判断)

## 技术笔记

- 内置策略写死代码(Rust const 或前端硬编码,非外置 JSON)。倾向前端 builtin store(类似 builtin bizTypes 但硬编码)。
- AutoGenStrategyDefine:code/name/paramDesc(Option<String>),比 BizTypeDefine 轻(无 supportedDataTypes/fields)。
- 策略有/无参数:有参数 paramDesc Some,无参数 paramDesc None。定义界面用 checkbox/switch 控制是否有参数,有则显示 paramDesc 输入。
- 字段编辑:选策略后,如果策略 paramDesc Some -> 显示 param 输入(placeholder=paramDesc);None -> 不显示 param。
- Java @GeneratedValue:对应策略 code。
