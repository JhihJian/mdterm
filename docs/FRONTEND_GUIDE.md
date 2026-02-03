# Mdterm 前端开发指南

本文档指导前端开发者如何集成 Mdterm 后端服务。

## Mdterm 提供什么

Mdterm 是一个将本地文件目录暴露为 Web 服务的后端应用，为前端应用提供以下能力：

### 核心能力

| 能力 | 说明 | 应用场景 |
|------|------|----------|
| **文件浏览** | 查询目录结构、文件列表、读取文件内容 | 构建文件管理器、文档浏览器 |
| **实时通知** | WebSocket 推送文件创建、修改、删除事件 | 协同编辑提示、文件变更提醒 |
| **远程终端** | 每个连接独立的交互式 PTY 会话 | Web IDE、在线终端、远程命令执行 |

### 特性

- **多目录管理** - 通过 Context 配置同时管理多个独立目录
- **智能过滤** - 自动应用 `.gitignore` 规则，只暴露 `.md` 文件
- **安全隔离** - 每个终端连接独立的 PTY，互不干扰
- **标准协议** - RESTful API + WebSocket，易于任何前端技术栈集成

### 典型应用场景

- 在线 Markdown 编辑器
- 文档知识库系统
- 代码浏览工具
- Web 终端模拟器
- 协同办公平台

---

## 目录

- [REST API](#rest-api)
- [WebSocket 协议](#websocket-协议)
- [集成示例](#集成示例)
- [最佳实践](#最佳实践)

---

## 服务地址

默认运行在 `http://127.0.0.1:8080`

（服务启动方式请参考项目 [README](../README.md)）

---

## REST API

所有 REST API 端点都在 `/api` 路径下。

### 基础响应格式

**成功响应：**
```json
{
  "field": "value"
}
```

**错误响应：**
```json
{
  "code": "ERROR_CODE",
  "message": "详细错误信息"
}
```

### 端点列表

#### 1. 健康检查

```http
GET /api/health
```

**响应：**
```json
{
  "status": "ok",
  "active_terminals": null,
  "connections": null
}
```

#### 2. 获取所有 Context

```http
GET /api/contexts
```

**响应：**
```json
[
  {
    "name": "docs",
    "path": "/home/user/documents",
    "description": "我的文档"
  },
  {
    "name": "notes",
    "path": "/home/user/notes",
    "description": "笔记目录"
  }
]
```

#### 3. 获取文件列表

```http
GET /api/:context/files?path=subdirectory
```

**路径参数：**
- `context` - context 名称

**查询参数：**
- `path` (可选) - 相对于 context 根目录的子路径

**响应：**
```json
[
  {
    "name": "README.md",
    "path": "docs/README.md",
    "size": 1024,
    "mtime": 1738555200,
    "is_dir": false
  },
  {
    "name": "guides",
    "path": "docs/guides",
    "size": 4096,
    "mtime": 1738555300,
    "is_dir": true
  }
]
```

**字段说明：**
| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | string | 文件/目录名 |
| `path` | string | 相对于 context 根目录的路径 |
| `size` | number | 字节大小 |
| `mtime` | number | 修改时间（Unix 时间戳） |
| `is_dir` | boolean | 是否为目录 |

**注意：**
- 只返回 `.md` 文件
- 隐藏文件（以 `.` 开头）会被过滤
- 应用 `.gitignore` 规则

#### 4. 获取目录树

```http
GET /api/:context/tree
```

**响应：**
```json
{
  "name": "root",
  "path": "",
  "children": [
    {
      "name": "README.md",
      "path": "README.md",
      "size": 1024,
      "mtime": 1738555200
    },
    {
      "name": "guides",
      "path": "guides",
      "children": [
        {
          "name": "tutorial.md",
          "path": "guides/tutorial.md",
          "size": 2048,
          "mtime": 1738555400
        }
      ]
    }
  ]
}
```

#### 5. 获取文件内容

```http
GET /api/:context/content?path=file.md
```

**查询参数：**
- `path` (必需) - 文件路径

**响应：**
```
Content-Type: text/markdown

# 文件标题

文件内容...
```

**错误状态码：**
- `404` - 文件不存在
- `403` - 文件被忽略规则过滤或非 .md 文件
- `500` - 服务器错误

---

## WebSocket 协议

Mdterm 提供两种 WebSocket 连接：

### 1. 文件变更通知

**连接地址：**
```
ws://host/api/:context/ws/notify
```

#### 服务端推送消息

**文件创建：**
```json
{
  "type": "created",
  "path": "docs/new-file.md",
  "mtime": 1738555500
}
```

**文件修改：**
```json
{
  "type": "modified",
  "path": "docs/existing.md",
  "mtime": 1738555600
}
```

**文件删除：**
```json
{
  "type": "deleted",
  "path": "docs/old.md"
}
```

#### 客户端行为

- 客户端需要处理 Ping/Pong（大多数 WebSocket 库自动处理）
- 连接断开时自动重连
- 收到事件后更新 UI

#### JavaScript 示例

```javascript
const notifyWs = new WebSocket('ws://127.0.0.1:8080/api/docs/ws/notify');

notifyWs.onmessage = (event) => {
  const data = JSON.parse(event.data);

  switch (data.type) {
    case 'created':
      console.log('文件创建:', data.path);
      onFileCreated(data);
      break;
    case 'modified':
      console.log('文件修改:', data.path);
      onFileModified(data);
      break;
    case 'deleted':
      console.log('文件删除:', data.path);
      onFileDeleted(data);
      break;
  }
};

notifyWs.onerror = (error) => {
  console.error('WebSocket 错误:', error);
};

notifyWs.onclose = () => {
  console.log('连接断开，3秒后重连...');
  setTimeout(() => reconnect(), 3000);
};
```

### 2. 终端会话

**连接地址：**
```
ws://host/api/:context/ws/terminal
```

每个 WebSocket 连接会创建一个独立的 PTY（伪终端）会话。

#### 连接流程

1. 客户端连接
2. 服务端发送握手消息
3. 双向通信开始

#### 握手消息（服务端 → 客户端）

```json
{
  "type": "handshake",
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "cols": 80,
  "rows": 24
}
```

保存 `session_id` 用于调试，每个连接有唯一 ID。

#### 输入消息（客户端 → 服务端）

**发送输入数据：**
```json
{
  "type": "input",
  "data": "bGFzCg=="
}
```

- `data` 是 Base64 编码的字节数据
- 例如：`ls\n` → Base64 → `bGFzCg==`

**调整终端大小：**
```json
{
  "type": "resize",
  "cols": 120,
  "rows": 30
}
```

#### 输出消息（服务端 → 客户端）

**终端输出：**
```json
{
  "type": "output",
  "data": "SGVsbG8gd29ybGQK"
}
```

- `data` 是 Base64 编码的终端输出

**会话结束：**
```json
{
  "type": "exit",
  "code": 0
}
```

#### JavaScript 示例（使用 xterm.js）

```javascript
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';

// 初始化 xterm.js
const terminal = new Terminal({
  cursorBlink: true,
  fontSize: 14,
  fontFamily: 'Monaco, Menlo, "Courier New", monospace'
});
const fitAddon = new FitAddon();
terminal.loadAddon(fitAddon);
terminal.open(document.getElementById('terminal'));
fitAddon.fit();

// 辅助函数：Base64 编码/解码
function base64Encode(str) {
  return btoa(unescape(encodeURIComponent(str)));
}

function base64Decode(str) {
  return decodeURIComponent(escape(atob(str)));
}

// 连接终端 WebSocket
const terminalWs = new WebSocket('ws://127.0.0.1:8080/api/docs/ws/terminal');

terminalWs.onopen = () => {
  console.log('终端连接已建立');
};

terminalWs.onmessage = (event) => {
  const msg = JSON.parse(event.data);

  switch (msg.type) {
    case 'handshake':
      console.log('会话 ID:', msg.session_id);
      // 调整终端大小匹配初始值
      terminal.resize(msg.cols, msg.rows);
      break;

    case 'output':
      // 解码 Base64 并写入终端
      const data = base64Decode(msg.data);
      terminal.write(data);
      break;

    case 'exit':
      console.log('会话结束，退出码:', msg.code);
      break;
  }
};

// 用户输入 → 发送到服务端
terminal.onData((data) => {
  if (terminalWs.readyState === WebSocket.OPEN) {
    terminalWs.send(JSON.stringify({
      type: 'input',
      data: base64Encode(data)
    }));
  }
});

// 终端大小变化 → 通知服务端
window.addEventListener('resize', () => {
  fitAddon.fit();
  if (terminalWs.readyState === WebSocket.OPEN) {
    terminalWs.send(JSON.stringify({
      type: 'resize',
      cols: terminal.cols,
      rows: terminal.rows
    }));
  }
});

// 清理
terminalWs.onclose = () => {
  console.log('终端连接已断开');
  terminal.dispose();
};
```

---

## 集成示例

### React + xterm.js 完整示例

```tsx
import React, { useEffect, useRef, useState } from 'react';
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import 'xterm/css/xterm.css';

interface MdtermTerminalProps {
  serverUrl: string;
  context: string;
}

export function MdtermTerminal({ serverUrl, context }: MdtermTerminalProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    if (!containerRef.current) return;

    // 初始化终端
    const terminal = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4'
      }
    });
    terminal.loadAddon(new FitAddon());
    terminal.open(containerRef.current);
    terminalRef.current = terminal;

    // 连接 WebSocket
    const ws = new WebSocket(
      `${serverUrl.replace('http', 'ws')}/api/${context}/ws/terminal`
    );

    ws.onopen = () => {
      setConnected(true);
    };

    ws.onmessage = (event) => {
      const msg = JSON.parse(event.data);

      if (msg.type === 'output') {
        const data = atob(msg.data);
        terminal.write(data);
      } else if (msg.type === 'handshake') {
        terminal.resize(msg.cols, msg.rows);
      }
    };

    ws.onclose = () => {
      setConnected(false);
    };

    // 用户输入
    terminal.onData((data) => {
      ws.send(JSON.stringify({
        type: 'input',
        data: btoa(unescape(encodeURIComponent(data)))
      }));
    });

    wsRef.current = ws;

    return () => {
      ws.close();
      terminal.dispose();
    };
  }, [serverUrl, context]);

  return (
    <div>
      <div className="status">
        状态: {connected ? '已连接' : '未连接'}
      </div>
      <div ref={containerRef} className="terminal-container" />
    </div>
  );
}
```

### Vue 3 文件浏览器示例

```vue
<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';

interface Context {
  name: string;
  path: string;
  description: string;
}

interface FileInfo {
  name: string;
  path: string;
  size: number;
  mtime: number;
  is_dir: boolean;
}

const serverUrl = 'http://127.0.0.1:8080';
const contexts = ref<Context[]>([]);
const currentContext = ref<string>('');
const currentPath = ref<string>('');
const files = ref<FileInfo[]>([]);
const loading = ref(false);
const fileContent = ref<string>('');
const selectedFile = ref<string>('');

// 获取所有 context
const fetchContexts = async () => {
  const res = await fetch(`${serverUrl}/api/contexts`);
  contexts.value = await res.json();
  if (contexts.value.length > 0) {
    currentContext.value = contexts.value[0].name;
    fetchFiles();
  }
};

// 获取文件列表
const fetchFiles = async () => {
  if (!currentContext.value) return;

  loading.value = true;
  const pathParam = currentPath.value ? `?path=${currentPath.value}` : '';
  const res = await fetch(
    `${serverUrl}/api/${currentContext.value}/files${pathParam}`
  );

  if (!res.ok) {
    console.error('获取文件列表失败:', await res.json());
    loading.value = false;
    return;
  }

  files.value = await res.json();
  loading.value = false;
};

// 获取文件内容
const fetchContent = async (filePath: string) => {
  const res = await fetch(
    `${serverUrl}/api/${currentContext.value}/content?path=${filePath}`
  );

  if (!res.ok) {
    console.error('获取文件内容失败:', await res.json());
    return;
  }

  fileContent.value = await res.text();
  selectedFile.value = filePath;
};

// 进入目录
const enterDirectory = (file: FileInfo) => {
  if (file.is_dir) {
    currentPath.value = currentPath.value
      ? `${currentPath.value}/${file.name}`
      : file.name;
    fetchFiles();
  } else {
    fetchContent(file.path);
  }
};

// 返回上级
const goUp = () => {
  if (!currentPath.value) return;
  const parts = currentPath.value.split('/');
  parts.pop();
  currentPath.value = parts.join('/');
  fetchFiles();
};

// 格式化文件大小
const formatSize = (bytes: number) => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
};

// 格式化时间
const formatTime = (timestamp: number) => {
  return new Date(timestamp * 1000).toLocaleString('zh-CN');
};

onMounted(() => {
  fetchContexts();
});
</script>

<template>
  <div class="file-browser">
    <!-- Context 选择 -->
    <div class="context-selector">
      <label>目录: </label>
      <select v-model="currentContext" @change="fetchFiles">
        <option v-for="ctx in contexts" :key="ctx.name" :value="ctx.name">
          {{ ctx.description || ctx.name }}
        </option>
      </select>
    </div>

    <!-- 文件列表 -->
    <div class="file-list">
      <div class="breadcrumb">
        <span v-if="currentPath" @click="goUp" class="back">..</span>
        <span class="current-path">{{ currentPath || '/' }}</span>
      </div>

      <div v-if="loading">加载中...</div>

      <table v-else>
        <thead>
          <tr>
            <th>名称</th>
            <th>大小</th>
            <th>修改时间</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="file in files"
            :key="file.path"
            @click="enterDirectory(file)"
            :class="{ directory: file.is_dir }"
          >
            <td>{{ file.name }}</td>
            <td>{{ file.is_dir ? '-' : formatSize(file.size) }}</td>
            <td>{{ formatTime(file.mtime) }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 文件内容 -->
    <div v-if="selectedFile" class="file-content">
      <h3>{{ selectedFile }}</h3>
      <pre>{{ fileContent }}</pre>
    </div>
  </div>
</template>

<style scoped>
.file-browser {
  padding: 20px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

.context-selector {
  margin-bottom: 20px;
}

.file-list table {
  width: 100%;
  border-collapse: collapse;
}

.file-list th,
.file-list td {
  padding: 8px;
  text-align: left;
  border-bottom: 1px solid #eee;
}

.file-list tr:hover {
  background-color: #f5f5f5;
  cursor: pointer;
}

.file-list .directory {
  font-weight: bold;
  color: #0066cc;
}

.back {
  color: #0066cc;
  cursor: pointer;
  margin-right: 10px;
}

.file-content {
  margin-top: 20px;
  padding: 15px;
  background-color: #f5f5f5;
  border-radius: 4px;
}

.file-content pre {
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
```

---

## 最佳实践

### 1. 错误处理

```javascript
async function fetchFileContent(context, path) {
  try {
    const response = await fetch(
      `http://127.0.0.1:8080/api/${context}/content?path=${path}`
    );

    if (!response.ok) {
      const error = await response.json();

      switch (response.status) {
        case 404:
          throw new Error('文件不存在');
        case 403:
          throw new Error('文件被忽略或不是 Markdown 文件');
        default:
          throw new Error(error.message || '未知错误');
      }
    }

    return await response.text();
  } catch (error) {
    console.error('获取文件内容失败:', error);
    throw error;
  }
}
```

### 2. WebSocket 重连

```javascript
class ReconnectingWebSocket {
  constructor(url, options = {}) {
    this.url = url;
    this.reconnectInterval = options.reconnectInterval || 3000;
    this.maxReconnectAttempts = options.maxReconnectAttempts || 10;
    this.reconnectAttempts = 0;
    this.onMessage = options.onMessage || (() => {});
    this.onOpen = options.onOpen || (() => {});
    this.onClose = options.onClose || (() => {});

    this.connect();
  }

  connect() {
    this.ws = new WebSocket(this.url);

    this.ws.onopen = () => {
      console.log('WebSocket 已连接');
      this.reconnectAttempts = 0;
      this.onOpen();
    };

    this.ws.onmessage = (event) => {
      this.onMessage(event);
    };

    this.ws.onclose = () => {
      console.log('WebSocket 连接断开');
      this.onClose();

      if (this.reconnectAttempts < this.maxReconnectAttempts) {
        this.reconnectAttempts++;
        console.log(`${this.reconnectInterval}ms 后尝试重连...`);
        setTimeout(() => this.connect(), this.reconnectInterval);
      }
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket 错误:', error);
    };
  }

  send(data) {
    if (this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(data);
    } else {
      console.warn('WebSocket 未连接，无法发送消息');
    }
  }

  close() {
    this.reconnectAttempts = this.maxReconnectAttempts; // 停止重连
    this.ws.close();
  }
}

// 使用
const notifyWs = new ReconnectingWebSocket('ws://127.0.0.1:8080/api/docs/ws/notify', {
  onMessage: (event) => {
    const data = JSON.parse(event.data);
    console.log('收到通知:', data);
  },
  reconnectInterval: 3000
});
```

### 3. Base64 工具函数

```javascript
/**
 * 将字符串编码为 Base64（正确处理 Unicode）
 */
export function base64Encode(str) {
  return btoa(unescape(encodeURIComponent(str)));
}

/**
 * 将 Base64 解码为字符串（正确处理 Unicode）
 */
export function base64Decode(str) {
  return decodeURIComponent(escape(atob(str)));
}

/**
 * 将 Uint8Array 编码为 Base64
 */
export function base64EncodeBytes(bytes) {
  const binString = Array.from(bytes, byte => String.fromCharCode(byte)).join('');
  return btoa(binString);
}

/**
 * 将 Base64 解码为 Uint8Array
 */
export function base64DecodeBytes(str) {
  const binString = atob(str);
  return Uint8Array.from(binString, char => char.charCodeAt(0));
}
```

### 4. 终端尺寸同步

```javascript
// 确保终端尺寸正确同步
function syncTerminalSize(terminal, ws) {
  const fitAddon = new FitAddon();
  terminal.loadAddon(fitAddon);

  // 初始适配
  fitAddon.fit();

  // 监听窗口大小变化
  const resizeObserver = new ResizeObserver(() => {
    const { cols, rows } = terminal;
    fitAddon.fit();

    // 如果尺寸变化，通知服务端
    if (terminal.cols !== cols || terminal.rows !== rows) {
      ws.send(JSON.stringify({
        type: 'resize',
        cols: terminal.cols,
        rows: terminal.rows
      }));
    }
  });

  resizeObserver.observe(terminal.element);

  return () => resizeObserver.disconnect();
}
```

### 5. 文件变更监听

```javascript
class FileWatcher {
  constructor(serverUrl, context, callbacks = {}) {
    this.wsUrl = `${serverUrl.replace('http', 'ws')}/api/${context}/ws/notify`;
    this.callbacks = {
      onCreated: callbacks.onCreated || (() => {}),
      onModified: callbacks.onModified || (() => {}),
      onDeleted: callbacks.onDeleted || (() => {})
    };

    this.connect();
  }

  connect() {
    this.ws = new WebSocket(this.wsUrl);

    this.ws.onmessage = (event) => {
      const data = JSON.parse(event.data);

      switch (data.type) {
        case 'created':
          this.callbacks.onCreated(data);
          break;
        case 'modified':
          this.callbacks.onModified(data);
          break;
        case 'deleted':
          this.callbacks.onDeleted(data);
          break;
      }
    };
  }

  close() {
    this.ws.close();
  }
}

// 使用
const watcher = new FileWatcher('http://127.0.0.1:8080', 'docs', {
  onCreated: (event) => {
    console.log('新文件:', event.path);
    // 刷新文件列表或显示通知
    showNotification(`新文件: ${event.path}`);
  },
  onModified: (event) => {
    console.log('文件已修改:', event.path);
    // 如果正在查看该文件，重新加载内容
    if (currentFile.value === event.path) {
      reloadFile(event.path);
    }
  },
  onDeleted: (event) => {
    console.log('文件已删除:', event.path);
    showNotification(`文件已删除: ${event.path}`);
  }
});
```

---

## TypeScript 类型定义

```typescript
// ========== 响应类型 ==========

interface HealthResponse {
  status: string;
  active_terminals: number | null;
  connections: number | null;
}

interface Context {
  name: string;
  path: string;
  description: string;
}

interface FileInfo {
  name: string;
  path: string;
  size: number;
  mtime: number;
  is_dir: boolean;
}

interface TreeNode {
  name: string;
  path: string;
  size?: number;
  mtime?: number;
  children?: TreeNode[];
}

interface ErrorResponse {
  code: string;
  message: string;
}

// ========== WebSocket 文件通知类型 ==========

type FileEvent = FileCreatedEvent | FileModifiedEvent | FileDeletedEvent;

interface FileCreatedEvent {
  type: 'created';
  path: string;
  mtime: number;
}

interface FileModifiedEvent {
  type: 'modified';
  path: string;
  mtime: number;
}

interface FileDeletedEvent {
  type: 'deleted';
  path: string;
}

// ========== WebSocket 终端类型 ==========

interface TerminalHandshake {
  type: 'handshake';
  session_id: string;
  cols: number;
  rows: number;
}

interface TerminalOutput {
  type: 'output';
  data: string; // Base64 编码
}

interface TerminalExit {
  type: 'exit';
  code: number;
}

type TerminalServerMessage = TerminalHandshake | TerminalOutput | TerminalExit;

interface TerminalInput {
  type: 'input';
  data: string; // Base64 编码
}

interface TerminalResize {
  type: 'resize';
  cols: number;
  rows: number;
}

type TerminalClientMessage = TerminalInput | TerminalResize;
```

---

## 故障排查

### 问题：WebSocket 连接失败

**检查清单：**
1. 确认服务正在运行：`curl http://127.0.0.1:8080/api/health`
2. 检查 URL 格式：`ws://127.0.0.1:8080/api/{context}/ws/...`
3. 检查 context 名称是否正确

### 问题：终端输出乱码

**原因：** Base64 编码/解码不正确

**解决：** 使用提供的工具函数，正确处理 Unicode 字符

### 问题：文件列表为空

**检查：**
1. 确认目录中有 `.md` 文件
2. 检查 `.gitignore` 是否过滤了文件
3. 查看服务端日志

---

## 相关资源

- **xterm.js 文档:** https://xtermjs.org/
- **WebSocket API:** https://developer.mozilla.org/en-US/docs/Web/API/WebSocket
- **GitHub 仓库:** https://github.com/JhihJian/mdterm

---

## 更新日志

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0.0 | 2025-02 | 初始版本 |
