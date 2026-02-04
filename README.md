# 开放题库服务端接口

该项目是开放题库前端和管理工具后台 api 实时接口服务.

## 环境说明

服务端使用的是 [Actix Web](https://actix.rs/) 框架.

### 启动

阅读 [main.rs](src/main.rs) 方法中的服务相关环境变量配置部分:

```
// 服务相关环境变量配置
#[derive(Deserialize)]
struct EnvConfig {
    database_url: String,
    server_host: String,
    server_port: u16,
    meta_path: String,
}
```

本地开发时请拷贝 [.env.example](.env.example) 文件为 `.env` 文件到同级目录位置并将 Key 对应的值更新为自己的环境配置， 该文件已在忽略文件中标记不会提交到代码库中.

### 数据库

数据库使用 [PostgreSQL](https://www.postgresql.org/), 各发行版安装方式请查看官网的指导, 部分发行版比如 Arch Linux 是要自己手动初始化数据库的, 其它发行版比如 Debian 可能安装完毕就可以使用, 具体请根据自己的开发环境决定并更改.

数据库的名字和用户等信息均在连接信息中, 因此不需要统一, SQL 语句见文件 [open_tiku.sql](open_tiku.sql) 内容, 复制或者导入即可创建表和索引信息, 文件中未关联任何数据库名.

如果要切换为其它数据库, model 层目前是写死的 `use sqlx::{FromRow, PgPool};` PgPool 所以没法直接兼容其它类型的数据库, 如果需要需要调整 model 内的定义.

考虑到查询并不复杂, 所以没有使用 ORM 框架, 而是使用了相对轻量的 [sqlx](https://crates.io/crates/sqlx), 本身支持了大部分类型的数据库, 因此如果要更换数据库原则上只需要调整 model 内的表连接池类型即可.

关于事务, 如果需要使用事务, 可参考该方法 [edit](src/service/textbook.rs), 要支持事务需要注意调整常规写法, 不需要事务不用写这么复杂

```
/// 修改记录
pub async fn update<'e, E>(
    executor: E,
    ...
) -> Result<Self, sqlx::Error>
where
    // 使用此约束可以同时接收 &PgPool 和 &mut Transaction
    E: Executor<'e, Database = Postgres>,
{

}

let db = &app_conf.get_ref().db;

// 开启事务
let mut tx = db.begin().await.map_err(|e| {
    error!("Error beginning transaction: {}", e);
    Error::new(ErrorKind::Other, "更新失败")
})?;

// 注意传参方式为 &mut *tx
let row = Textbook::update(
    &mut *tx,
    ...
)

// 最后提交即可
tx.commit().await.map_err(|e| {
    error!("Error committing transaction: {}", e);
    Error::new(ErrorKind::Other, "更新失败")
})?;
```

#### 常量

[meta.rs](src/constant/meta.rs) 中有这么一行

```
/// 图片访问 api 前缀, 由 nginx 决定
pub const IMAGE_READ_PREFIX: &str = "api";
```

由于没有提供文件服务, 因此图片等资源是跟随服务存储在本机, 只能通过接口自行读取文件, 如果你配置了 nginx 代理需要关注该常量的值

### 构建

生产环境包构建脚本细节查看 [build.sh](build.sh).

如果本地环境是 Linux 可以本地直接构建即可, 如果本地不是 Linux 环境目前配置了 Github Actions 工作流, main 分支合并完成后手动触发构建即可, 构建完成后是 草稿 状态, 需要进行编辑对应的标签重新发布后才可见.

### 部署

目前仅提供了基于 Debian 的部署脚本 [deploy.sh](deploy.sh), 详情可查看该文件内容.

部署时需要先指定线上 .env 文件, 类似

```
source .env && sh deploy.sh start -v v0.0.1-beta
```
