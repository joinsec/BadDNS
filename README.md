# BadDNS

BadDNS 是一款使用 Rust 开发的使用公共 DNS 服务器进行多层子域名探测的极速工具。

本软件只做初步探测，请使用者遵守《中华人民共和国网络安全法》，勿将 BadDNS 用于非授权的测试，莲隐科技/雾隐实验室不负任何连带法律责任。

### 设计思路

- 使用随机字符串作为子域名，使用内置数个公共 DNS 服务器解析，以此来生成泛解析白名单，为后面排除泛解析做铺垫
- 读取 `subdomain` 字典用于生成待查询目标，读取 `depth` 字典用于判断是否进行深层子域名查询
- 使用 `TCP` 进行解析查询
- 检查解析结果是否存在于白名单，如果存在于白名单则抛弃结果
- 检查 `sub` 字段是否存在于 `depth` 字典中，如果存在则进行下一个深度的域名探测，不存在则不进行后续处理

### 深度探测

以 `sub` 为 `api` 探测为例，该探测有结果并且不存在于泛解析白名单及 `api` 存在于 `depth` 字典，则进行下一级子域名探测；如果该探测没有相应的结果则不再进行 `api` 下一子域的探测。
这样解决市面上的爆破工具大量字典傻傻的问题
同时，希望大家能踊跃补充 `depth` 字典

### 命令行参数说明

| 参数      | 说明          | 默认值|
| ------------- |:-------------:|:--------:|
| -h     | 输出帮助          | None|
| -v     | 输出日志信息      | None|
| -V     | 输出版本信息      | None|
| -t     | 指定目标文件      | None|   
| -d     | 指定depth字典文件 | depthdict.txt|
| -l     | 设置子域深度      | 1|
| -m     | 设置内存占用率    | 0.5(50%)|
| -o     | 指定结果保存文件   |baddns-output.json|
| -s     | 指定subdomain字典文件| domaindict-170W.txt|
| -w     | 设置线程池大小|500|

### 入门示例

- 使用默认配置

    `./baddns -t target.txt -s domaindict-170W.txt -d depthdict.txt`

- 配置8个线程和结果保存至 `baddns-outputs-8.json`

    `./baddns -t target.txt -w 8 -o baddns-outputs-8.json -s domaindict-170W.txt -d depthdict.txt`

- 配置二级子域深度探测

    `./baddns -t target.txt -s domaindict-170W.txt -d depthdict.txt -l 2`

### 推荐运行环境（防止各种诡异bug ^_^）

- 该版本支持 `Linux X64-86` 平台
- 入门配置**1核2GB内存**
- 推荐配置**8核16GB及以上内存**
- 推荐使用无限制带宽的VPS供应商
- 需要配置Linux调优执行 `ulimit -n 655350`

### 演示视频
[BadDNS example](https://youtu.be/OU0Sq7zt_iI)

### 源码编译

1. 安装Rust并配置交叉编译环境

    - 安装Rust 
    
        `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
    
    - 安装`Linux-x86_64`工具链
    
        `rustup target add x86_64-unknown-linux-musl`
  
   可参见[官方教程](https://www.rust-lang.org/learn/get-started)

2. 编译
    
    - git clone 源码
  
        `git clone https://github.com/joinsec/BadDNS.git`
     
    - 进入项目目录执行编译命令
    
      - 交叉编译
        
        `cargo build --target x86_64-unknown-linux-musl --release`
        
      - 普通编译
      
        `cargo build --release`
    
    - 可执行文件位于`target`目录下

        
    
