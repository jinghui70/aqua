# 打包发布

aqua 桌面应用的构建、打包与分发说明。

---

## 1. 产物

- **macOS**:`.dmg` 安装包
- **Windows**:`.exe`(NSIS 安装包,免管理员权限)
- 不做 Linux 包、不做代码签名、不做自动更新。

安装包内置 `connector.jar`(Java 连接器,~5MB),用户自备 JDK 17+ 即可连接 Oracle/信创/H2。

---

## 2. 构建前置

| 依赖 | 用途 | 说明 |
|------|------|------|
| Node.js + pnpm | 前端构建 + 编排脚本 | `pnpm install` |
| Rust toolchain | 编译 src-tauri 壳 + aqua-core | `rustup` |
| JDK 17+ | 构建 connector.jar(Maven) | 打包机必备 |
| Maven | 构建 connector.jar | `mvn -q package` |

> 终端用户的机器只需 JDK 17+(仅连接 Oracle/信创/H2 时);MySQL/PostgreSQL 走 Rust native,免 Java。

---

## 3. 一条命令打包

```sh
pnpm tauri build
```

`beforeBuildCommand` 会自动按序执行:

1. `pnpm build:connector` — `mvn package` 产 `connector/target/connector.jar`,复制到 `src-tauri/resources/connector.jar`(打包 resource)。
2. `pnpm --filter @aqua/app build` — 前端产物输出到 `app/dist`。

随后 Tauri 编译 release 二进制并产出安装包:
- macOS:`src-tauri/target/release/bundle/dmg/aqua_<version>_aarch64.dmg`
- Windows:`src-tauri/target/release/bundle/nsis/aqua_<version>_x64-setup.exe`

> 跨平台不可交叉构建:macOS 只能产 dmg,Windows 包需在 Windows 机器上执行 `pnpm tauri build`。

---

## 4. connector.jar 与外置驱动(两条路径,不可混用)

| 路径 | 内容 | 定位方式 | 读写 |
|------|------|----------|------|
| bundle resource | `connector.jar`(连接器本体) | `resource_dir`(打包内置) | 只读 |
| `app_data_dir/drivers/` | 外置 JDBC 驱动 jar(Oracle/达梦等) | `app_data_dir` | 用户可写 |

- **connector.jar**:随安装包内置,Rust 端通过 `resource_dir` 解析绝对路径,dev 与打包后一致。用户无需关心。
- **外置 JDBC 驱动**:用户通过应用内「配置 → 数据库配置」安装(对应 `install_driver` 命令),jar 被复制到 `app_data_dir/drivers/` 并登记到 `databases.json`。
  - macOS:`~/Library/Application Support/com.aqua.app/drivers/`
  - Windows:`%APPDATA%\com.aqua.app\drivers\`
- H2 驱动已 shade 进 `connector.jar`,无需用户额外安装;Oracle/信创库需用户自备并安装对应 JDBC 驱动。

---

## 5. JDK 17+ 检测

连接 Java 数据源(Oracle/信创/H2)前,aqua 会执行 `java -version` 检测:

- **缺失 java** 或 **版本 < 17**:连接被拦截,弹出明确提示(需 JDK 17+,请安装并配置 JAVA_HOME/PATH),不崩溃、不静默失败。
- 检测结果首次缓存,避免每次连接重复 spawn。
- MySQL/PostgreSQL 走 Rust native,**不触发** java 检测,无需 Java。

---

## 6. 分发与首次打开

未签名,首次打开需手动绕过系统警告:

- **macOS**:双击 dmg 拖入应用后,首次启动若提示"无法打开"或"已损坏",**右键点击应用 → 打开** → 确认。仅首次需要。
- **Windows**:运行 setup.exe 后,SmartScreen 可能提示"未识别的应用",点击 **"更多信息" → "仍要运行"**。

分发方式:GitHub Release(发版时 CI 自动上传,见 §8)或内部共享。无应用内自动更新。

---

## 7. 开发模式注意事项

- `src-tauri/resources/connector.jar` 是构建产物(已 gitignore)。`tauri dev` 启动时通过 `beforeDevCommand` 自动执行 `pnpm build:connector:if-missing`:**仅在 jar 不存在时构建一次**(fresh clone 首次启动会跑一次 Maven,之后启动只做一次文件存在性检查,近乎零开销)。
- connector 源码(`connector/`)改动后,dev 不会自动重建;需手动 `pnpm build:connector` 全量重建后再 `tauri dev`。
- 仅 `tauri build` 的 `beforeBuildCommand` 每次全量重建 connector.jar(确保产物最新)。

---

## 8. CI 自动打包与发版

发版由 GitHub Actions(`.github/workflows/release.yml`)自动打包,本地 `bumpp` 触发。

### 发版

```sh
pnpm release
```

`bumpp` 交互选版本(patch/minor/major/pre-release),自动同步 4 处版本号(根 + `app/package.json`、`tauri.conf.json`、`Cargo.toml` `[workspace.package]`,经 `scripts/version-sync.mjs`),再 `commit + tag vX.Y.Z + push`。推送 tag 触发 CI:macOS + Windows 并行构建,产出 dmg/exe 上传到 GitHub Release(附自动 release notes)。

### CI 触发方式

- `push tag v*`:两平台并行构建 -> 上传 dmg + nsis exe 到 GitHub Release。
- `workflow_dispatch`(Actions 页手动):只构建留 artifact 供下载测试,不建 Release。

> 产物版本一致性:`tauri.conf.json`(dmg/exe 文件名)、`Cargo.toml`(二进制)、`package.json`(bumpp 入口)三者由 `version-sync.mjs` 发版时强制同步。
