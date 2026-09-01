# 打包版 GUI 进程找不到 JDK:java 解析逻辑修复

## Goal

修复打包后的应用在 GUI 启动环境下无法定位 JDK 的问题:本地 `pnpm dev` 可以正常使用 JDBC 功能(连接/导入/数据集导出),打包安装后报"未检测到 Java 运行时"或"无法解析 Java 版本"。

## 背景 / 根因

`crates/aqua-core/src/driver/jdbc.rs` 的 `java_command()` 使用 `Command::new("java")`,完全依赖进程自身的 PATH 查找 java:

- `pnpm dev`:从终端 shell 启动,继承用户完整 PATH(含 homebrew/JDK bin),能找到真 JDK。
- 打包版(macOS .app):从 Finder/Dock 启动,进程环境由 launchd 提供,PATH 仅有 `/usr/bin:/bin:/usr/sbin:/sbin`。此时解析到的是 Apple 的 `/usr/bin/java` stub:
  - 纯 homebrew 安装的 JDK 未注册到 `/Library/Java/JavaVirtualMachines/`,stub 输出 `No Java runtime present, requesting install.`(无可解析版本行)→ 走到"无法解析 Java 版本"分支(`jdbc.rs:230`)。
  - 无 JDK 时 stub 同样失败 → "未检测到 Java 运行时"分支(`jdbc.rs:215`)。

所有 JDBC 功能共用唯一的检测点 `check_java_once()`(`jdbc.rs:203`),连接、反解导入、数据集导出均受影响。

## Requirements

1. `java_command()` 不再直接依赖 PATH,改为按优先级解析出 java 可执行文件绝对路径:
   - **JAVA_HOME/bin/java**(三平台通用;当前报错文案承诺了 JAVA_HOME 但代码从未读取,需补上)
   - **平台专属候选路径**:
     - macOS:执行 `/usr/libexec/java_home` 获取真实 JDK Home;homebrew 兜底 `/opt/homebrew/opt/openjdk*/bin/java`、`/opt/homebrew/bin/java`、`/usr/local/opt/openjdk*/bin/java`、`/usr/local/bin/java`
     - Windows:常见安装目录 `C:\Program Files\Java\*\bin\java.exe`、`C:\Program Files\Eclipse Adoptium\*\bin\java.exe` 等
     - Linux:`/usr/lib/jvm/*/bin/java`
   - **PATH fallback**:以上都失败时退回 `Command::new("java")`(保持 dev 环境行为)
2. 解析结果缓存:沿用现有 `OnceCell` 结构,一次进程生命周期内只解析一次;检测失败不缓存(用户装好 JDK 重试可重新检测,保持现有语义)。
3. 报错信息增强:
   - "无法解析 Java 版本"分支的报错中带上 `java -version` 的原始输出(当前只在 log::warn 里,用户看不到)。
   - 报错中带上实际解析到的 java 路径(或"未解析到,使用 PATH")。
4. Windows 现有行为不回退:CREATE_NO_WINDOW(`jdbc.rs:21`)、verbatim 路径 strip(`jdbc.rs:184`)等既有逻辑保持不变。
5. `/usr/libexec/java_home` 执行需要设置输出捕获与超时,失败静默跳过进入下一候选。

## 非目标

- 不做 JDK 自动下载/捆绑(jre bundle 方案另行评估,本次只修查找)。
- 不改 connector.jar 及其协议。
- 不改前端;报错文案仍由后端透传。

## Acceptance Criteria

- [ ] 单元测试:新增解析顺序的测试(mock 候选路径存在性/`java -version` 输出解析,沿用 `jdbc.rs:465` 现有测试风格)。
- [ ] macOS 打包版安装后,连接/导入/数据集导出 JDBC 功能可用(homebrew JDK 场景)。
- [ ] 设置 `JAVA_HOME` 后打包版可用(三平台通用路径)。
- [ ] dev 环境(`pnpm dev`)行为不回退。
- [ ] 无 JDK 时报错文案包含排查所需信息(实际尝试的路径 + `java -version` 原始输出)。
- [ ] `cargo test -p aqua-core` 通过;Windows 编译不受影响(`#[cfg(windows)]` 分支语法正确,可通过 cargo check --target 交叉验证或人工 review)。

## Notes

- 用户实测报错:"无法解析 Java 版本(连接 JDBC 数据源需 JDK 17+,请检查 JAVA_HOME/PATH 配置)",出现在数据集导出,即 `jdbc.rs:230` 分支。
- 检测逻辑仅此一处(`check_java_once`),不存在多处 java 判断。
- 轻量级任务,PRD-only。
