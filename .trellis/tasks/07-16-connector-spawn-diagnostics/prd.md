# 诊断 Windows connector spawn 连接失败(加子进程日志)

## 背景 / 问题

Windows 发版后配置数据源、测试连接,前端显示"连接失败 + 一堆乱码 + connector.jar 路径 + 乱码"。

已排查确认(见对话):
- 同一台 Windows(代码页 936/GBK, JDK 17.0.17): **手动** `echo {...} | java -jar connector.jar` 返回 `{status:"ok"}`,一切正常。
- connector.jar 打包路径正确、文件存在、`resolve` 拼路径与实际位置对得上。
- 应用(Tauri GUI)去 spawn 时**失败**,并弹出黑窗口(说明 java 确实被启动)。
- 乱码本质:java 进程按系统代码页(GBK)输出,Rust 端 `jdbc.rs` 用 `from_utf8_lossy` 按 UTF-8 解析 → 中文乱码,遮蔽了真实错误。

核心矛盾:**同机、同 jar,手动成功、应用失败**。差异点未知(疑似 GUI 进程 PATH / 工作目录 / stdin 写入时序 / 或其它),已连猜三轮,决定停止推测。

## 目标

给 Rust spawn connector 子进程的路径加**落文件的诊断日志**,一次发版即可拿到确凿现场,定位"手动成功 vs 应用失败"的真实差异。同时消除 GBK 乱码对错误信息的遮蔽,让报错可读。

**本 task 不承诺修复连接失败本身**(根因待日志确认);产出的是可观测性 + 编码可读性两个确定性改进。

## Requirements

1. 引入能在 Windows 用户机落文件的日志(GUI 无 console,黑窗口一闪即逝,日志必须写文件)。
   - `aqua-core` 纯逻辑核心不得依赖 tauri;用 `log` facade emit,由 `src-tauri` 外壳装 sink。
2. 在 `crates/aqua-core/src/driver/jdbc.rs` 的 spawn 路径(`call` + `check_java_once`)记录完整现场:
   - 完整 argv(`java -jar <connector_path>`)、`drivers_dir`、request JSON(**password 必须脱敏**)。
   - spawn 成功/失败(失败含 io error)、进程 exit code。
   - stdout / stderr 原始字节:**同时按 UTF-8(lossy)和 GBK 两种解码打印**,保证日志本身不再乱码,且能看清 java 吐的中文。
3. 修复错误信息可读性:Windows 上 java 子进程输出按系统代码页(GBK)解码后再回传前端,不再硬 UTF-8。
   - 独立次要 bug 一并修:`jdbc.rs` 在 exit≠0 时只读 stderr,但 Java 错误经 `writeError` 写到 **stdout** 的 JSON。应优先解析 stdout 的 `{error:...}`,避免真实报错丢失。

## 非目标 (Out of Scope)

- 修复连接失败根因本身(等日志证据,后续 task)。
- 消除黑窗口(`CREATE_NO_WINDOW`):本轮**保留**黑窗口作为"java 已启动"证据,定位后另行处理。
- 改动 connector(Java 端)代码。

## Acceptance Criteria

- [ ] `aqua-core` 通过 `log` facade emit,不新增 tauri 依赖;`cargo build`/`cargo test` 通过。
- [ ] `src-tauri` 装好日志 sink,Windows 下日志落到确定的文件路径(在 prd/journal 记录该路径)。
- [ ] 一次测试连接会在日志文件产出:完整 argv、drivers_dir、脱敏 request、exit code、stdout/stderr 的 UTF-8+GBK 双解码。
- [ ] 前端展示的 connector 报错在中文 Windows 上不再乱码(GBK 正确解码)。
- [ ] exit≠0 时优先回传 stdout `{error:...}` 的真实原因。
- [ ] 现有测试不回归(`parse_java_major_version` 等)。
