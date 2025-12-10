# 开放题库服务端接口

## 说明

服务端使用的是 [Actix Web](https://actix.rs/) 框架.

## 启动

阅读 main.rs 方法中的命令行参数部分: 

```
/// 命令行参数结构
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// 元数据路径 命令行实际输入时下划线需要转化为中横线, 例如 open-tiku-api --meta-path /home/zhangguangxun/Public/open-tiku-meta
    #[clap(long, default_value = "/home/zhangguangxun/Public/open-tiku-meta")]
    meta_path: PathBuf,

    /// 监听地址
    #[clap(long, default_value = "127.0.0.1")]
    host: String,

    /// 监听端口
    #[clap(long, default_value = "8080")]
    port: u16,
}
```

本地开发时为了方便 IDE 快速启动, 自行更改 meta_path 的默认值 default_value 为自己开发环境的 meta 元数据根目录.

命令行启动服务, 形如:

```
$ ./target/debug/open-tiku-api --meta-path /home/zhangguangxun/Public/open-tiku-meta
```

## 部署

目前仅提供了基于 Debian 的部署脚本 deploy.sh, 详情可查看该文件内容.