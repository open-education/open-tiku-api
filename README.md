# 开放题库服务端接口

该项目是 *开放题库* 前端 api 实时接口服务.

## 环境说明

服务端使用 [Actix Web](https://actix.rs/) 框架.

### 启动

阅读 [conf.rs](src/app/conf.rs) 方法中的服务相关环境变量配置部分:

```rust
// 服务相关环境变量配置
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: PgPool,
}
```

本地开发时请拷贝 [config.example.toml](.config.example.toml) 文件为 `config.toml` 文件到同级目录位置并将 Key 对应的值更新为自己的环境配置，
该文件已在忽略文件中标记不会提交到代码库中, 一些账户相关的信息等需要从其它渠道等获取, 或者咨询项目维护者.

### 数据库

数据库使用 [PostgreSQL](https://www.postgresql.org/), 各发行版安装方式请查看官网的指导, 部分发行版比如 Arch Linux
是要自己手动初始化数据库的, 其它发行版比如 Debian 可能安装完毕就可以使用, 具体请根据自己的开发环境决定并更改.

数据库的名字和用户等信息均在连接信息中, 因此不需要统一, SQL 语句见文件 [open_tiku.sql](open_tiku.sql) 内容,
复制或者导入即可创建表和索引信息, 文件中未关联任何数据库名.

如果要切换为其它数据库, model 层目前是写死的 `use sqlx::{FromRow, PgPool};` PgPool 所以没法直接兼容其它类型的数据库,
如果需要请调整 model 内的定义.

考虑到查询并不复杂, 所以没有使用 ORM 框架, 而是使用了相对轻量的 [sqlx](https://crates.io/crates/sqlx), 本身支持了大部分类型的数据库,
因此如果要更换数据库原则上只需要调整 model 内的表连接池类型即可.

关于事务,
可参考这类方法 [edit](src/service/textbook.rs), [tx_insert](src/service/question.rs), [tx_batch_insert](src/service/question.rs)
等不同方式的事务写法，

由于没有提供文件服务, 因此图片等资源是跟随服务存储在本机, 只能通过接口自行读取文件, 如果你配置了 caddy 等代理需要关注该常量的值

### 静态文件目录

静态文件目录, 比如线上的 `/var/www/meta` 目录, 首次需要创建共享组并添加用户, 本地完全可以直接给予 `sudo chmod -R 777 /var/www/meta/*` 权限

```bash
# 1. 创建共享组并添加用户
sudo groupadd www-media

# 把所有需要写入权限的用户都加入这个组
sudo usermod -a -G www-media zhangguangxun # 部署后端应用的用户
sudo usermod -a -G www-media caddy # caddy 服务

# 2. 设置目录权限
# 目录所有者设为你的登录用户，所属组设为 www-media
sudo chown -R zhangguangxun:www-media /var/www/meta

# 权限：用户和组可读写，其他人只读
sudo chmod -R 775 /var/www/meta

# 设置 setgid，让新建文件自动继承 www-media 组
sudo chmod g+s /var/www/meta
```

### 构建

生产环境包构建脚本细节查看 [build.sh](build.sh).

如果本地环境是 `Linux` 可以本地直接构建即可, 如果本地不是 `Linux` 环境目前配置了 `Github Actions` 工作流, main
分支合并完成后手动触发构建即可, 构建完成后是 草稿 状态, 需要进行编辑对应的标签重新发布后才可见.

线上环境的 GLIBC 版本号如下

```bash
zhangguangxun@VM-0-4-debian:~/open-tiku-api$ ldd --version
ldd (Debian GLIBC 2.36-9+deb12u7) 2.36
Copyright (C) 2022 Free Software Foundation, Inc.
This is free software; see the source for copying conditions.  There is NO
warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
Written by Roland McGrath and Ulrich Drepper.
zhangguangxun@VM-0-4-debian:~/open-tiku-api$ 
```

所以本地的 `GLIBC` 版本号如果高过 `2.36` 则编译完成的二进制不能启动成功；此时就需要借助 `.github` 文件夹中的工作流让 `GitHub` 平台帮忙编译, 
工作流配置的环境是 `ubuntu-22.04` 其实这个版本的系统已经是不维护的状态, 目前暂无升级操作系统的计划, 因此知晓这一步骤即可.

### 部署

目前仅提供了基于 Debian 的部署脚本 [deploy.sh](deploy.sh), 详情可查看该文件内容.

首次部署时需要先指定线上 `config.toml` 文件, 后续部署项目会自动在可执行文件同级目录查找该文件

```bash
sh deploy.sh start -v v0.0.1-beta
```

### 公私钥生成

学生账户登录密码加密方式公私钥生成, 首次部署时需要, 其它场景去环境中拷贝即可

```bash
# 生成私钥 PKCS#8 格式
openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out private_key.pem
```

```bash
# 从私钥中提取公钥 PKCS#8 / DER 格式
openssl rsa -pubout -in private_key.pem -out public_key.pem
```
