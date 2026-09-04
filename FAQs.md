# FAQs

该文档用于记录开发中遇到的问题和解决方法, 每个问题后面都记录了操作系统名称, 请不要盲目去执行其中的命令, 确认可行后再执行,
避免带来更多未知的问题

## 1 数据库排序版本不匹配 (Arch, Debian)

```log
database "open_tiku_test" has a collation version mismatch
```

操作系统升级时, 其核心库 `glibc` (GNU C 库) 的版本可能发生变更, `PostgreSQL` 在进行字符串排序或比较时, 会依赖操作系统底层的
`glibc` 库来实现具体的排序规则 (`collation`)

`glibc` 在不同版本间, 其对同一语言环境 (如 `en_US.UTF-8`) 的排序逻辑可能会发生改变

数据库创建时, `PostgreSQL` 会记录下当时 `glibc` 的版本号 比如 (`2.42`) 当操作系统升级后, `glibc` 更新为新版本 (`2.44`),
`PostgreSQL` 在启动或执行查询时检测到版本不一致, 就会发出该警告

这不仅是一个信息提示, 更是一个数据一致性警告. 如果 `glibc` 的排序规则真的发生了变化, 那么数据库中所有建立在文本字段上的索引
(`B-tree` 索引) 可能仍然是按照旧的排序顺序排列的

这会导致：

- 查询结果错误: 使用索引的 `ORDER BY` 或范围查询可能返回不正确的顺序或遗漏数据
- 数据约束破坏: 依赖索引的唯一约束可能失效, 导致数据逻辑上的损坏

修复方式

### 1.1 检查有哪些库受影响

```sql
SELECT datname, datcollversion, pg_database_collation_actual_version(oid) AS current_os_version
FROM pg_database;
```

### 1.2 重建该库的所有索引

持有排他锁, 需在维护窗口执行

```sql
REINDEX DATABASE open_tiku_test;
```

### 1.3 刷新版本号

```sql
ALTER DATABASE open_tiku_test REFRESH COLLATION VERSION;
```

### 1.4 系统数据库

比如 `postgresql` 和 `template1`, 具体根据第 1 步骤中的输出做处理

`postgres` 只是一个默认的连接维护库, 本身没有特殊模板作用

`template0` 的 `datcollversion` 是空的 (`| 空值`), 这不是不匹配, 而是正常现象

`template0` 是 `PostgreSQL` 的 *原始* 基础模板, 它强制使用 `C` 语言环境 (不依赖操作系统的 `glibc`), 因此它没有记录版本号

不要对 `template0` 执行刷新操作, 因为它是只读的, 无法连接写入, 并且它本身并没有问题

`template1` 是 `PostgreSQL` 创建所有新数据库的默认模板, 未来任何新建的数据库 (比如 `CREATE DATABASE new_db;`)
都会自动继承这个过时的版本号

操作系统数据库可能需要登录管理员账户

```bash
sudo -u postgres psql
```

刷新涉及到的数据库版本号

```sql
ALTER DATABASE postgres REFRESH COLLATION VERSION;
ALTER DATABASE template1 REFRESH COLLATION VERSION;
```

## 2. 代码扫描

rust 的部分代码编译器不会自动优化, 需要使用扫描工具分析后根据建议修复.

### 2.1 静态代码扫描 Clippy

```bash
cargo clippy -- -W clippy::perf
```

Clippy 是 Rust 官方自带的静态分析 (Linting)神兵利器. 它内部专门有一个 perf (性能)分类, 里面集成了数百条规则,
专门用来阻止你写出像"多余的 let 阻断优化", "不必要的 .clone ()"这样的低性能代码
