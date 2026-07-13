# 数据源持久化

## Goal

数据源配置持久化到项目目录 `.dbconfig.json`,密码用 AES-256-GCM 加密,替换当前纯内存态的 datasource store。

## Requirements

- **落盘位置**:`.dbconfig.json` 与项目文件同目录(已在 `.gitignore`)。
- **文件结构**:`{ sources: { <name>: { dialect, host, port, user, password, database, schema? } } }`。`password` 存加密串,其余明文。
- **密码加密**:AES-256-GCM。密钥为 32 字节随机值,首次运行生成并存用户数据目录 `key`(权限 600);后续复用。密文格式 `base64(nonce ‖ ciphertext+tag)`。
- **密钥路径归属**:aqua-core 不感知平台目录;src-tauri 用 Tauri path API 解析 app data dir 后把 key 文件路径传入 aqua-core。
- **无项目路径时**:数据源仅存内存;保存/打开项目后,`.dbconfig.json` 随项目目录联动读写。
- **前端 store 改造**:`add/update/remove` 后落盘;打开项目时加载。空密码不加密(存空串)。
- **测试连接不变**:沿用现有 `testConnection` command。

## Acceptance Criteria

- [ ] 新增/编辑/删除数据源并保存项目后,同目录出现 `.dbconfig.json`,`password` 字段为密文。
- [ ] 重新打开同一项目,数据源列表恢复,密码可解密用于连接。
- [ ] key 文件缺失时自动生成;存在时复用,解密成功。
- [ ] 手工损坏密文或换机器(key 不同)时,解密失败返回明确错误,不 panic。
- [ ] aqua-core 单测覆盖:加解密 roundtrip、错误密文报错、dbconfig 读写 roundtrip。
- [ ] clippy 0 warning,pnpm build 通过。

## Notes

- 密钥策略与设计文档的"机器特征派生"不同:改为用户目录随机密钥(更稳、可控)。已与用户确认。见 design.md §密钥。
- aqua-core 依赖 `aes-gcm = "0.10"` 已在 Cargo.toml。
