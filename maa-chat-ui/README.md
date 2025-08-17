# MAA 智能助手前端

基于 React + Vite 的聊天界面，用于与 MAA 智能控制后端进行交互。

## 功能特点

- 现代化 UI 设计，支持亮色/暗色主题
- 实时连接状态监控
- 集成 Qwen API 进行智能对话
- 完全响应式设计
- 基于 Vite 的快速开发体验

## 快速开始

### 安装依赖
```bash
npm install
```

### 开发模式
```bash
npm run dev
```
访问 http://localhost:3000

### 生产构建
```bash
npm run build
```

### 预览构建
```bash
npm run preview
```

## 环境要求

- Node.js 18+
- MAA 后端服务运行在 localhost:8080
- Qwen API 密钥配置

## 项目结构

```
maa-chat-ui/
├── public/
│   └── assets/
│       └── maa-logo.png      # MAA 官方看板娘头像
├── index.html                # 主页面
├── main.jsx                  # React 应用入口
├── vite.config.js           # Vite 配置
└── package.json             # 依赖配置
```

## 技术栈

- React 19
- Vite 5
- CSS 变量 (支持主题切换)
- Qwen API
- MAA Function Calling