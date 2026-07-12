# MySQL native 驱动实现

## 背景

aqua v2 移植路线 P0 最后一步。MySQL native 驱动是 Driver trait 的第一个具体实现,验证 trait 设计的完整性。

- 依赖: Driver trait 已定义
- 驱动: mysql_async (已在 Cargo.toml)
- 设计规范: `docs/architecture.md` §4 内置方言

## 目标

实现 MySQL native 驱动,支持连接测试、表列出、列反解、索引反解。

**包含**:
- MysqlDriver 结构体
- Driver trait 所有方法实现
- MySQL 物理类型 → aqua 逻辑类型反解
- 连接管理(一次性连接,无连接池)

## 范围(不含)

- 数据查询(query_rows,数据集模块用)
- 连接池(一次性连接即可)
- SSL/SSH 连接(用户自保证可达)

## 验收标准

**实现**:
- [ ] `driver/mysql.rs` MysqlDriver 实现
- [ ] 实现 test_connection
- [ ] 实现 list_tables
- [ ] 实现 get_columns (含类型反解)
- [ ] 实现 list_indexes
- [ ] 更新 factory.rs 注册 MySQL 驱动

**测试**:
- [ ] 单元测试: 类型反解逻辑
- [ ] 集成测试: 连接真实 MySQL(可选,CI 配置)

**质量**:
- [ ] `cargo test -p aqua-core` 通过
- [ ] `cargo clippy -p aqua-core -- -D warnings` 无 warning

## 约束

- 使用 mysql_async
- 一次性连接,执行完即关闭
- 类型反解遵循 design.md §4.1 规则
- 错误统一转换为 DriverError

## 参考

- mysql_async 文档: https://docs.rs/mysql_async/
- MySQL 类型映射: design.md §4.1
