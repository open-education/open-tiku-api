# 开放题库服务端接口

## 说明

服务端使用的是 [Actix Web](https://actix.rs/) 框架.

## 启动

阅读 [main.rs](src/main.rs) 方法中的命令行参数部分:

```

```

本地开发时为了方便 IDE 快速启动, 自行更改 meta_path 的默认值 default_value 为自己开发环境的 meta 元数据根目录.

命令行启动服务, 形如:

```
$ ./target/debug/open-tiku-api --meta-path /home/zhangguangxun/Public/open-tiku-meta
```

## 构建

生产环境包构建脚本细节查看 [build.sh](build.sh).

## 部署

目前仅提供了基于 Debian 的部署脚本 [deploy.sh](deploy.sh), 详情可查看该文件内容.