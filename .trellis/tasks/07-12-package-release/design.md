# 打包发布 - 技术设计

## 架构边界
- **aqua-core(纯逻辑)**:`JdbcDriver` 已持有 `connector_path` + `drivers_dir`;本任务加 `check_java()`(`std::process`,不依赖 tauri)。
- **src-tauri(壳)**:用 Tauri resource API 算 connector.jar 绝对路径(dev/打包一致),传 `create_driver`;commands 已注入 `AppHandle`。
- **connector(Maven)**:`mvn package` 产 `connector/target/connector.jar`(shade jackson+h2,main-class `com.aqua.connector.Main`);Oracle/信创 JDBC 驱动外置 `drivers/`。
- **构建**:pnpm script `build:connector` 接 Maven 构建 + 复制 jar 到 `src-tauri/resources/`;`beforeBuildCommand` 调用。

## 现状基线(已就绪,勿重复实现)
- `create_driver(config, drivers_dir)` + `JdbcDriver::new(config, connector_path, drivers_dir)` + `call()` 传 `driversDir`:**全链路已实现**。
- 外置 JDBC 驱动目录 `app_data_dir/drivers/`(`database.rs:drivers_dir()` 解析+建目录,用户可写):**已实现**。
- import.rs 3 个 command(test_connection / import_from_db / list_tables)已注入 `AppHandle<R>` 并传 `drivers`:**已实现**。
- 图标 `src-tauri/icons/icon.icns` + `icon.ico` + 各尺寸 png:**已存在**(`tauri icon` 已跑过)。
- connector 构建:`mvn package` 产 `connector/target/connector.jar`,H2 shade 内置:**已就绪**。

## 数据流

### connector.jar 路径定位(R2 核心,待实现)
- 打包:connector.jar 作为 Tauri bundle resource(`bundle.resources`),打包后位于 `resource_dir()`。
- dev:connector.jar 放 `src-tauri/resources/connector.jar`(`build:connector` 脚本复制),dev 时 resource_dir 指向此处,统一。
- src-tauri commands(已注入 `AppHandle`):`app.path().resource_dir()?.join("connector.jar")` 算绝对路径,传 `create_driver`。
- `create_driver` 现签名 `(config, drivers_dir)`,追加第3参 `connector_path: &str`(mysql/pg 忽略),传 `JdbcDriver::new`(已支持该参)。替换硬编码 `"connector.jar"`(`factory.rs:30`)。
- **两条路径不可混**:connector.jar 走 `resource_dir`(打包内置,只读);外置 JDBC 驱动走 `app_data_dir/drivers/`(用户可写)。

### java 检测(R3,待实现)
- `JdbcDriver` 加 `check_java()`:`std::process::Command::new("java").arg("-version")`,解析 stderr 版本,<17 或缺失报 `DriverError`(清晰提示"需 JDK 17+,请安装并配置 JAVA_HOME/PATH")。
- `check_java` 在 `call` 前(首次调缓存,避免每次 spawn 开销)。mysql/pg 不触发。
- 放 aqua-core(`std::process`,不依赖 tauri)。

### Maven 构建集成(R2,待实现)
- pnpm script `build:connector`:`cd connector && mvn -q package && cp target/connector.jar ../src-tauri/resources/connector.jar`(跨平台用 pnpm script 封装,避免 shell 差异)。
- `tauri.conf.json build.beforeBuildCommand`:现 `pnpm --filter @aqua/app build`,前置 `pnpm build:connector`。
- `bundle.resources` 加 `"resources/connector.jar"`。
- `src-tauri/resources/connector.jar` 是构建产物,gitignore(见 implement C3)。

### 平台 bundle(R1,待实现)
- macOS:dmg;Windows:nsis(轻量,免管理员;企业部署后续可加 msi)。
- `bundle.targets` 按平台(macOS dmg,Windows nsis),或保留 "all" 各平台构建时自动选。
- 图标 icns/ico 已存在,仅需 `bundle.icon` 从 `["icons/icon.png"]` 改为含 `.ico`/`.icns`。
- 不签名:macOS 未公证(右键打开),Windows 未签名(SmartScreen 警告,文档说明)。

## 兼容性
- dev:connector.jar 在 `src-tauri/resources/`,resource_dir 定位一致。
- 打包后:connector.jar 在 bundle resource_dir,同 API 定位。
- 无旧数据(首次打包)。

## Tradeoffs
| 决策 | 取舍 |
|---|---|
| connector.jar 内置 resource | 开箱即用 ✅ / 体积 +几 MB ❌ |
| JDBC 驱动外置 drivers/(用户自备) | 无版权/体积小 ✅ / 用户要自己下驱动 ❌ |
| java 检测 spawn `java -version` | 准确 ✅ / 连接开销(可缓存)❌ |
| 不签名 | 零成本 ✅ / 首次警告 ❌ |
| nsis(vs msi) | 轻量免管理员 ✅ / 企业部署可能要 msi ❌ |

## Decisions
1. **drivers/ 机制代码已就绪**(`app_data_dir/drivers/`),本任务仅补文档(E1);connector.jar 走 resource_dir,两者分离。
2. **Windows 用 nsis**:轻量免管理员,企业部署需 msi 时后续再加。
3. **Tauri 2 resource_dir API**:implement 阶段验证 dev/打包路径一致性。

## 验证命令
- connector 构建:`cd connector && mvn package` -> `target/connector.jar`
- 前端:`pnpm --filter @aqua/app build`
- 打包:`pnpm tauri build`(macOS 产 dmg,Windows 产 nsis)
- 编译:`cargo check -p aqua-core && cargo check -p aqua`、`cd app && pnpm exec vue-tsc --noEmit`
