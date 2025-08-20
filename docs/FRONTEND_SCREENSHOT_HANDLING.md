# 前端截图处理指南

## 问题背景

截图功能返回大量base64数据（可达几十KB），如果保存到聊天历史会导致：
1. 下次AI调用时发送大量数据，触发422错误
2. 前端界面卡顿，消息加载缓慢
3. 浏览器内存占用过高

## 解决方案

### 1. 后端响应格式

**纯截图响应** (screenshot_only: true):
```json
{
  "choices": [{
    "message": {
      "role": "assistant",
      "content": "截图已完成！",
      "tool_calls": [...]
    }
  }],
  "screenshot": {
    "status": "success", 
    "screenshot_id": "screenshot_20250820_123456",
    "base64_data": "iVBORw0KGgoAAAANSUhEUgAA...",
    "file_size": 123456,
    "timestamp": "2025-08-20 12:34:56"
  },
  "screenshot_only": true
}
```

**混合操作响应** (有截图+其他操作):
```json
{
  "choices": [{
    "message": {
      "role": "assistant", 
      "content": "任务执行完成！截图已单独处理。",
      "tool_calls": [...]
    }
  }]
  // 无screenshot字段，截图已在工具调用中处理
}
```

### 2. 前端处理逻辑

```javascript
// 聊天响应处理
async function handleChatResponse(response) {
  const data = await response.json();
  
  // 检查是否为纯截图响应
  if (data.screenshot_only && data.screenshot) {
    // 截图响应：不保存到历史，直接显示
    displayScreenshot(data.screenshot);
    
    // 添加一个简化的历史记录（不含图片数据）
    addToHistory({
      role: "assistant",
      content: "📸 截图已完成",
      timestamp: new Date().toISOString(),
      screenshot_ref: data.screenshot.screenshot_id // 只保存引用
    });
    
    return;
  }
  
  // 普通消息：正常保存到历史
  const message = data.choices[0].message;
  addToHistory({
    role: message.role,
    content: message.content,
    timestamp: new Date().toISOString(),
    tool_calls: message.tool_calls
  });
  
  displayMessage(message);
}

// 显示截图
function displayScreenshot(screenshot) {
  const screenshotElement = document.createElement('div');
  screenshotElement.className = 'screenshot-display';
  screenshotElement.innerHTML = `
    <div class="screenshot-header">
      <span>📸 MAA截图 - ${screenshot.timestamp}</span>
      <button onclick="downloadScreenshot('${screenshot.screenshot_id}')">下载</button>
    </div>
    <img src="data:image/png;base64,${screenshot.base64_data}" 
         alt="MAA截图" 
         style="max-width: 100%; border-radius: 8px;"/>
    <div class="screenshot-footer">
      <small>文件大小: ${formatFileSize(screenshot.file_size)}</small>
    </div>
  `;
  
  chatContainer.appendChild(screenshotElement);
  scrollToBottom();
}

// 发送消息时过滤历史
function sendMessage(content) {
  // 获取历史消息，但排除包含大量数据的消息
  const history = chatHistory
    .filter(msg => {
      // 排除截图引用消息或超长消息
      return !msg.screenshot_ref && msg.content.length < 10000;
    })
    .slice(-10); // 只取最近10条
  
  const payload = {
    messages: [...history, {
      role: "user",
      content: content
    }]
  };
  
  fetch('/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload)
  })
  .then(handleChatResponse)
  .catch(handleError);
}
```

### 3. 消息历史管理

```javascript
class ChatHistoryManager {
  constructor() {
    this.messages = [];
    this.maxMessages = 50;
    this.maxMessageLength = 10000;
  }
  
  addMessage(message) {
    // 检查消息长度
    if (message.content.length > this.maxMessageLength) {
      console.warn('消息过长，可能包含图片数据，不保存到历史');
      return;
    }
    
    // 添加到历史
    this.messages.push({
      ...message,
      id: Date.now(),
      timestamp: new Date().toISOString()
    });
    
    // 保持历史长度
    if (this.messages.length > this.maxMessages) {
      this.messages = this.messages.slice(-this.maxMessages);
    }
    
    // 保存到本地存储（不包含图片数据）
    this.saveToStorage();
  }
  
  getHistoryForAI() {
    // 返回用于AI的历史消息（进一步过滤）
    return this.messages
      .filter(msg => !msg.screenshot_ref) // 排除截图引用
      .filter(msg => msg.content.length < 5000) // 严格长度限制
      .slice(-8); // 只取最近8条
  }
  
  clearLargeMessages() {
    // 清除可能包含大数据的消息
    this.messages = this.messages.filter(msg => 
      msg.content.length < 5000 && !msg.content.includes('base64')
    );
    this.saveToStorage();
  }
}
```

### 4. 错误处理

```javascript
function handleAPIError(response) {
  if (response.error === 'message_too_long') {
    // 提示用户清理历史
    showNotification('消息历史过长，正在自动清理...', 'warning');
    chatHistory.clearLargeMessages();
    return;
  }
  
  if (response.error === 'message_contains_large_data') {
    // 强制重置对话
    showNotification('检测到大量数据，已重置对话', 'info');
    chatHistory.clear();
    return;
  }
}
```

## 实施检查清单

- [ ] 检测 `screenshot_only` 字段
- [ ] 截图响应不保存到聊天历史
- [ ] 发送消息时过滤大数据
- [ ] 添加消息长度验证
- [ ] 实现历史清理功能
- [ ] 添加错误处理机制

## 测试场景

1. **纯截图**: 发送"截图"，确认响应不进入历史
2. **截图+操作**: 发送"截图并刷1-7"，确认正常处理
3. **历史验证**: 连续截图后发送普通消息，确认无422错误
4. **长度验证**: 手动发送超长消息，确认被拦截

这样就能完全避免截图数据污染聊天历史，从根本上解决422错误问题！