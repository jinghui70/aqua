# Driver trait 定义与工厂

## 背景

aqua v2 移植路线 P0 第三步。Driver trait 是数据库连接层的统一抽象,支撑导入(import)功能。

- 设计规范: `docs/architecture.md` §4 两类数据库支持机制
- 现状: generators(DDL/Java)已完成,Driver trait 待定义

## 目标

定义 Driver trait 统一接口,实现工厂模式,为 MySQL/PostgreSQL native 驱动和 JDBC 驱动提供统一抽象。

**包含**:
- Driver trait 定义: test_connection/list_tables/get_columns/list_indexes
- 元数据类型: ColumnMeta/IndexMeta
- 工厂函数: create_driver(config)
- DbConfig 配置结构

## 范围(不含)

- 具体驱动实现(MySQL/PG/JDBC 由后续任务实现)
- 数据查询(query_rows 接口定义但不实现)
- 连接池(一次性连接即可)

## 验收标准

**实现**:
- [ ] `driver/mod.rs` 模块声明
- [ ] `driver/types.rs` DbConfig/ColumnMeta/IndexMeta 类型
- [ ] `driver/trait.rs` Driver trait 定义
- [ ] `driver/factory.rs` create_driver 工厂函数

**设计**:
- [ ] trait 方法: async fn,返回 Result<T, DriverError>
- [ ] ColumnMeta 包含: name/data_type/length/precision/scale/nullable/is_key/default_value/comment
- [ ] IndexMeta 包含: name/fields/unique
- [ ] DbConfig 包含: dialect/host/port/user/password/database/schema

**文档**:
- [ ] trait 文档注释
- [ ] 使用示例(伪代码)

## 约束

- 异步 trait (async fn)
- 错误类型统一为 DriverError
- 反解结果返回 aqua 逻辑类型(DataType),不返回物理类型
- 工厂返回 Box<dyn Driver> (trait object)

## 参考

- architecture.md §4 Driver trait 收敛
- legacy: 无直接对应(原实现在 TS connector,现重新设计)
