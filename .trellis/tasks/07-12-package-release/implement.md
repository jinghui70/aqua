# 打包发布 - 执行计划

## 现状基线(已就绪,勿重做)
- drivers_dir 全链路(factory 参数 / jdbc.rs 传 driversDir / database.rs:drivers_dir() / import.rs 注入 AppHandle):✅
- 图标 icns/ico/png:✅(`src-tauri/icons/`)
- connector `mvn package` 产 connector.jar(shade jackson+h2):✅

## 实现顺序

### 阶段 A:connector.jar 路径定位(R2)
- [ ] A1 `aqua-core/driver/factory.rs`:`create_driver` 加第3参 `connector_path: &str`(mysql/pg 忽略),传 `JdbcDriver::new`,替换硬编码 `"connector.jar"`(factory.rs:30)。
- [ ] A2 `src-tauri/commands/import.rs`:3 个 command 算 `app.path().resource_dir()?.join("connector.jar")` 绝对路径,传 `create_driver(config, drivers, &connector_path)`(AppHandle 已注入、drivers 已传,只补 connector_path)。
- [ ] A3 `aqua-core/tests/integration_db.rs`:现有 `create_driver(cfg, None)` 调用补第3参(占位 `"connector.jar"`)。
- [ ] A4 验证:`cargo check -p aqua-core && cargo check -p aqua`。

### 阶段 B:java 检测(R3)
- [ ] B1 `aqua-core/driver/jdbc.rs`:加 `check_java()`(spawn `java -version`,解析版本 >=17),失败报 `DriverError` 清晰提示。
- [ ] B2 `JdbcDriver::call` 前调 `check_java`(首次缓存)。
- [ ] B3 验证:`cargo test -p aqua-core`。

### 阶段 C:connector 构建集成(R2)
- [ ] C1 根 `package.json` 加 script `build:connector`:mvn package + cp jar 到 `src-tauri/resources/connector.jar`(跨平台)。
- [ ] C2 `tauri.conf.json`:`bundle.resources` 加 `resources/connector.jar`;`beforeBuildCommand` 前置 `pnpm build:connector`;`bundle.icon` 补 `.ico`/`.icns`。
- [ ] C3 `src-tauri/resources/connector.jar` 是否 gitignore(产物,不提交二进制)。
- [ ] C4 验证:本地 `pnpm build:connector`,确认 jar 产出。

### 阶段 D:平台 bundle(R1)
- [ ] D1 `tauri.conf.json` bundle:macOS dmg,Windows nsis(targets 按平台或 "all")。
- [ ] D2 验证:macOS `pnpm tauri build` 产 dmg;Windows 在 Win 机器/CI 产 nsis。

### 阶段 E:drivers/ 文档(代码已就绪)
- [ ] E1 文档:用户放外置 JDBC 驱动到 `app_data_dir/drivers/`(应用内 install_driver command 已支持,文档说明用法)。

### 阶段 F:文档与分发(R4)
- [ ] F1 文档:用户需装 JDK 17+、放外置驱动、macOS/Win 首次打开绕过警告。
- [ ] F2 构建说明:一条命令打包(`pnpm tauri build`)。

### 阶段 G:整体验证
- [ ] G1 `cargo check -p aqua-core && cargo check -p aqua` + `cd app && pnpm exec vue-tsc --noEmit`。
- [ ] G2 手动:macOS 打包 dmg,安装,连接 H2(内置驱动)走通;无 java 时报清晰提示。
- [ ] G3 Windows 打包配置正确性自查(交叉构建不可行;Windows 实测由用户在 Win 环境完成)。

## 验证命令
- 后端:`cargo check -p aqua-core && cargo check -p aqua`
- 前端:`cd app && pnpm exec vue-tsc --noEmit`
- connector:`cd connector && mvn package`
- 打包:`pnpm tauri build`

## 风险点 / 回滚
- **Tauri 2 resource_dir API**:implement 验证写法,dev/打包路径一致。
- **beforeBuildCommand 跨平台**:macOS/Win shell 差异,用 pnpm script 封装。
- **Windows 打包**:macOS 无法交叉构建 Windows Tauri;本机只验 macOS + tauri.conf 配置正确,Windows 实测由用户在 Win 环境完成。
- **drivers/ 版权**:不打包驱动,用户自备。

## start 前检查
- [x] prd.md / design.md / implement.md 齐备。
- [x] open questions(drivers/ 纳入?✅ 代码已就绪;Windows 包格式 ✅ nsis)已确认。
- [ ] 用户 review + `task.py start`。
