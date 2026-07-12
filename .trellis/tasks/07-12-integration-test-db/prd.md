# 数据库集成测试 (MySQL/PG)

## 背景
aqua v2 核心代码完成,需验证 MySQL/PG native 驱动真实可用。当前只有单元测试,未连真实库。

## 目标
Docker compose 启 MySQL/PG 容器,集成测试验证 Driver trait 全链路 + import_from_db。

## 包含
- docker-compose.yml(MySQL 8 + PostgreSQL 16,固定端口)
- 集成测试(#[ignore] 手动触发)
- test_connection / list_tables / get_columns / list_indexes / import_from_db
- 往返: DDL 建表 -> 导入 -> 比对 Project

## 验收标准
- [ ] docker-compose.yml(MySQL:3306 / PG:5432,root/root,aqua_test 库)
- [ ] tests/integration_db.rs(#[ignore] 测试)
- [ ] MySQL + PG 全链路验证
- [ ] 测试运行步骤文档

## 约束
- 测试 #[ignore],默认不跑
- 手动: docker compose up -d && cargo test -- --ignored
- 建表用各自驱动库,保持 Driver trait 纯净
