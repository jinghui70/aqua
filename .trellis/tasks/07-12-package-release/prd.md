# 打包发布

## Goal
产出 macOS、Windows 可分发安装包(Tauri bundle + 内置 connector.jar),用户自备 JDK 17+ 即可运行 Oracle/信创/H2 连接。早期阶段:不签名、不自动更新,手动分发。

## Background
- Tauri 2.x,当前 bundle targets "all",仅打包 `builtin-biztypes.json` resource(`src-tauri/tauri.conf.json`)。
- connector 为 Maven 项目(`connector/pom.xml`),`JdbcDriver` 通过 `java -jar connector.jar` 子进程通信(`crates/aqua-core/src/driver/jdbc.rs`),一次性 stdin JSON -> stdout JSON。
- **现状缺陷**:`connector_path` 硬编码 `"connector.jar"`(`factory.rs:43`),相对路径,打包后定位不到 jar;`java` 仅在 spawn 失败时报"需 JDK 17+",无主动版本检测。
- MySQL/PG 走 Rust native,无需 Java;Oracle/信创/H2 走 JDBC connector,需 JDK 17+。

## Requirements

### R1 平台与 bundle
- 目标平台:macOS(dmg)、Windows(msi 或 nsis)。不做 Linux。
- Tauri bundle 配置产出两平台安装包。

### R2 connector.jar 集成
- connector Maven 构建产出 jar,打包进 Tauri bundle(resource)。
- Rust 端定位 bundle 内 connector.jar 绝对路径(dev 与打包后均可用),替换硬编码相对路径。
- 构建流程纳入:Tauri build 前自动构建 connector.jar(Maven package)。

### R3 Java 运行时检测
- 连接 Java 数据源(Oracle/信创/H2)前,检测 java(JAVA_HOME / PATH)+ 版本 >=17。
- 缺失/版本不足时,明确提示用户安装 JDK 17+(不静默失败、不崩溃)。
- 不打包 JRE,维持用户自备。

### R4 分发
- 手动分发(GitHub release / 内部共享),无应用内自动更新。
- 暂不签名:文档说明 macOS 右键打开、Windows SmartScreen 绕过。

## Acceptance Criteria
- [ ] macOS 产出 dmg,Windows 产出安装包(msi/nsis)。
- [ ] 安装包内含 connector.jar;连接 Oracle/H2 时 Rust 能定位 jar 并调通。
- [ ] 无 java 或版本 <17 时,连接 Java 数据源报清晰提示,不崩溃。
- [ ] 一条命令产出安装包(含 connector.jar Maven 构建)。
- [ ] 文档说明:用户需装 JDK 17+、两平台首次打开绕过警告。

## Out of Scope
- Linux 包。
- 精简 JRE 打包(用户自备 JDK)。
- 代码签名/公证。
- 应用内自动更新。
