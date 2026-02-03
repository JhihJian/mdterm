# Mdterm

将文件目录通过 HTTP API 和 WebSocket 暴露为服务的 Rust 应用。

## 功能

- 通过 context 配置映射目录路径
- REST API 查询文件列表、树状结构、内容
- 支持 `.gitignore` 文件过滤
- WebSocket 实时文件变更通知
- 交互式终端会话（每连接独立 PTY）

## 快速开始

```bash
# 克隆仓库
git clone git@github.com:JhihJian/mdterm.git
cd mdterm

# 创建配置文件
cp mdterm.toml.example mdterm.toml

# 编辑配置（设置你要暴露的目录）
vim mdterm.toml

# 运行
cargo run

# 或使用发布版本（更快）
cargo build --release
./target/release/mdterm
```

服务启动后访问 `http://127.0.0.1:8080`

## 配置

创建 `mdterm.toml`:

```toml
[server]
host = "127.0.0.1"
port = 8080

[[contexts]]
name = "docs"
path = "/home/user/documents"
description = "我的文档"
command = "/bin/bash"
```

## API

### REST

- `GET /api/health` - 健康检查
- `GET /api/contexts` - 列出 contexts
- `GET /api/:context/files?path=xxx` - 文件列表
- `GET /api/:context/tree` - 目录树
- `GET /api/:context/content?path=xxx` - 文件内容

### WebSocket

- `ws://host/api/:context/ws/notify` - 文件变更通知
- `ws://host/api/:context/ws/terminal` - 终端会话

## 终端协议

### 服务端 -> 客户端

```json
{"type":"handshake","session_id":"uuid","cols":80,"rows":24}
{"type":"output","data":"base64编码的输出"}
{"type":"exit","code":0}
```

### 客户端 -> 服务端

```json
{"type":"input","data":"base64编码的输入"}
{"type":"resize","cols":120,"rows":30}
```
