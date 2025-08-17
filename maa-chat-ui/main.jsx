import React, { useState, useEffect, useRef } from 'react'
import { createRoot } from 'react-dom/client'

// 现代化的MAA聊天组件 - 基于 assistant-ui 设计风格
const MAAChat = () => {
  const [messages, setMessages] = useState([
    {
      id: 1,
      role: 'assistant',
      content: '您好！我是MAA智能助手，可以帮您控制明日方舟自动化助手进行各种游戏操作。\n\n您可以尝试说：\n• "帮我截个图" - 获取当前游戏画面\n• "获取MAA状态" - 查看系统运行状态\n• "帮我做日常" - 自动执行日常任务\n• "查看我的干员" - 显示已识别的干员信息'
    }
  ])
  const [input, setInput] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [isConnected, setIsConnected] = useState(false)
  const messagesEndRef = useRef(null)

  // 检查MAA连接
  useEffect(() => {
    const checkConnection = async () => {
      try {
        const response = await fetch('http://localhost:8080/health')
        if (response.ok) {
          setIsConnected(true)
          updateStatus('connected')
        } else {
          setIsConnected(false)
          updateStatus('disconnected')
        }
      } catch (error) {
        setIsConnected(false)
        updateStatus('disconnected')
      }
    }

    checkConnection()
    const interval = setInterval(checkConnection, 5000)
    return () => clearInterval(interval)
  }, [])

  // 滚动到底部
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  // 更新状态
  const updateStatus = (status) => {
    const statusEl = document.getElementById('status')
    const infoEl = document.getElementById('info')
    
    if (statusEl && infoEl) {
      statusEl.className = `status ${status}`
      switch (status) {
        case 'connected':
          statusEl.innerHTML = '<div class="status-dot"></div>MAA后端已连接'
          infoEl.textContent = '可以开始对话'
          break
        case 'disconnected':
          statusEl.innerHTML = '<div class="status-dot"></div>MAA后端未连接'
          infoEl.textContent = '请确保后端服务运行在 localhost:8080'
          break
        case 'loading':
          statusEl.innerHTML = '<div class="status-dot"></div>处理中...'
          infoEl.textContent = '正在执行MAA操作'
          break
      }
    }
  }

  // 获取MAA工具
  const getMAATools = async () => {
    try {
      const response = await fetch('http://localhost:8080/tools')
      if (response.ok) {
        const data = await response.json()
        return data.tools || []
      }
    } catch (error) {
      console.error('获取MAA工具失败:', error)
    }
    return []
  }

  // 通过后端代理调用AI聊天
  const callAIChat = async (messages, tools) => {
    const response = await fetch('http://localhost:8080/chat', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        messages: messages.map(msg => ({
          role: msg.role,
          content: msg.content
        })),
        tools: tools,
        system_prompt: '你是MAA（明日方舟自动化助手）的智能控制助手。用户可以用自然语言向你描述想要执行的操作，你需要调用相应的MAA工具来完成。请用友好、简洁的中文回复。'
      })
    })

    if (!response.ok) {
      throw new Error(`后端AI聊天错误: ${response.status}`)
    }

    return await response.json()
  }

  // 执行MAA函数
  const callMAAFunction = async (functionName, args) => {
    const response = await fetch('http://localhost:8080/call', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        function_call: {
          name: functionName,
          arguments: args
        }
      })
    })

    if (!response.ok) {
      throw new Error(`MAA调用失败: ${response.status}`)
    }

    return await response.json()
  }

  // 发送消息
  const sendMessage = async () => {
    if (!input.trim() || isLoading || !isConnected) return

    const userMessage = input.trim()
    setInput('')
    setIsLoading(true)
    updateStatus('loading')

    // 添加用户消息
    const newMessages = [...messages, {
      id: Date.now(),
      role: 'user',
      content: userMessage
    }]
    setMessages(newMessages)

    try {
      // 获取工具列表
      const tools = await getMAATools()
      
      // 调用AI分析
      const response = await callAIChat(newMessages, tools)
      
      if (response.choices && response.choices[0]) {
        const choice = response.choices[0]
        
        // 检查是否需要调用工具
        if (choice.message.tool_calls && choice.message.tool_calls.length > 0) {
          const toolCall = choice.message.tool_calls[0]
          const functionName = toolCall.function.name
          const args = JSON.parse(toolCall.function.arguments)
          
          // 显示正在执行
          setMessages(prev => [...prev, {
            id: Date.now() + 1,
            role: 'assistant',
            content: `🔧 正在执行 ${functionName}\n\n\`\`\`json\n${JSON.stringify(args, null, 2)}\n\`\`\``
          }])
          
          // 执行MAA操作
          const result = await callMAAFunction(functionName, args)
          
          // 显示结果
          let resultText = '✅ 执行完成！'
          if (result.result) {
            if (typeof result.result === 'string') {
              resultText = `✅ ${result.result}`
            } else {
              resultText = `✅ 执行成功\n\n\`\`\`json\n${JSON.stringify(result.result, null, 2)}\n\`\`\``
            }
          }
          
          setMessages(prev => [...prev, {
            id: Date.now() + 2,
            role: 'assistant',
            content: resultText
          }])
          
          // AI的额外回复
          if (choice.message.content) {
            setMessages(prev => [...prev, {
              id: Date.now() + 3,
              role: 'assistant',
              content: choice.message.content
            }])
          }
        } else {
          // 直接回复
          setMessages(prev => [...prev, {
            id: Date.now() + 1,
            role: 'assistant',
            content: choice.message.content || '我理解了您的需求，但暂时无法执行相关操作。如果您需要执行MAA操作，请尝试更具体的指令。'
          }])
        }
      } else {
        setMessages(prev => [...prev, {
          id: Date.now() + 1,
          role: 'assistant',
          content: '抱歉，我没有理解您的意思。您可以尝试说：\n• "帮我截个图"\n• "获取MAA状态"\n• "查看我的干员"'
        }])
      }
    } catch (error) {
      console.error('处理消息失败:', error)
      setMessages(prev => [...prev, {
        id: Date.now() + 1,
        role: 'assistant',
        content: `❌ 抱歉，处理您的请求时出现错误：${error.message}`
      }])
    } finally {
      setIsLoading(false)
      updateStatus(isConnected ? 'connected' : 'disconnected')
    }
  }

  const handleKeyPress = (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      sendMessage()
    }
  }

  return (
    <>
      <div className="chat-interface">
        {/* 消息列表 */}
        <div className="messages-container">
          <div className="messages-list">
            {messages.map((message) => (
              <div key={message.id} className={`message-group ${message.role}`}>
                <div className="message-avatar">
                  {message.role === 'assistant' ? (
                    <img 
                      src="/assets/maa-logo.png" 
                      alt="MAA"
                      className="avatar-image"
                    />
                  ) : (
                    <div className="user-avatar">您</div>
                  )}
                </div>
                <div className="message-content">
                  <div className={`message-bubble ${message.role}`}>
                    {message.content}
                  </div>
                </div>
              </div>
            ))}
            <div ref={messagesEndRef} />
          </div>
        </div>
        
        {/* 输入区域 */}
        <div className="input-container">
          <div className="input-wrapper">
            <div className="input-field">
              <input
                type="text"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyPress={handleKeyPress}
                placeholder="输入您的指令，我会智能理解并执行..."
                disabled={!isConnected || isLoading}
                className="message-input"
              />
              <button
                onClick={sendMessage}
                disabled={!isConnected || isLoading || !input.trim()}
                className={`send-button ${(!isConnected || isLoading || !input.trim()) ? 'disabled' : ''}`}
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <line x1="22" y1="2" x2="11" y2="13"></line>
                  <polygon points="22,2 15,22 11,13 2,9"></polygon>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>

      <style jsx>{`
        .chat-interface {
          height: 100%;
          display: flex;
          flex-direction: column;
          background: var(--background);
        }

        .messages-container {
          flex: 1;
          overflow: hidden;
          display: flex;
          flex-direction: column;
        }

        .messages-list {
          flex: 1;
          overflow-y: auto;
          padding: 1.5rem;
          display: flex;
          flex-direction: column;
          gap: 1.5rem;
          scroll-behavior: smooth;
        }

        .message-group {
          display: flex;
          align-items: flex-start;
          gap: 0.75rem;
          max-width: 100%;
        }

        .message-group.user {
          flex-direction: row-reverse;
        }

        .message-avatar {
          flex-shrink: 0;
          width: 32px;
          height: 32px;
          margin-top: 0.25rem;
        }

        .avatar-image {
          width: 100%;
          height: 100%;
          border-radius: 50%;
          object-fit: cover;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          border: 2px solid var(--border);
        }

        .user-avatar {
          width: 100%;
          height: 100%;
          border-radius: 50%;
          background: linear-gradient(135deg, #3b82f6, #8b5cf6);
          display: flex;
          align-items: center;
          justify-content: center;
          color: white;
          font-size: 0.75rem;
          font-weight: 600;
          box-shadow: 0 2px 8px rgba(59, 130, 246, 0.3);
        }

        .message-content {
          flex: 1;
          min-width: 0;
        }

        .message-bubble {
          padding: 0.875rem 1.125rem;
          border-radius: 1.125rem;
          font-size: 0.9rem;
          line-height: 1.5;
          white-space: pre-wrap;
          word-wrap: break-word;
          max-width: fit-content;
          transition: all 0.2s ease;
          position: relative;
        }

        .message-bubble.user {
          background: linear-gradient(135deg, #3b82f6, #8b5cf6);
          color: white;
          margin-left: auto;
          box-shadow: 0 4px 12px rgba(59, 130, 246, 0.25);
        }

        .message-bubble.assistant {
          background: var(--card);
          color: var(--card-foreground);
          border: 1px solid var(--border);
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
        }

        .input-container {
          padding: 1.5rem;
          border-top: 1px solid var(--border);
          background: var(--background);
          backdrop-filter: blur(8px);
        }

        .input-wrapper {
          max-width: 800px;
          margin: 0 auto;
        }

        .input-field {
          display: flex;
          align-items: center;
          background: var(--card);
          border: 1px solid var(--border);
          border-radius: 1.5rem;
          padding: 0.5rem 0.75rem 0.5rem 1.25rem;
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
          transition: all 0.2s ease;
          position: relative;
        }

        .input-field:focus-within {
          border-color: #3b82f6;
          box-shadow: 0 4px 12px rgba(59, 130, 246, 0.15), 0 0 0 3px rgba(59, 130, 246, 0.1);
        }

        .message-input {
          flex: 1;
          border: none;
          outline: none;
          background: transparent;
          font-size: 0.95rem;
          color: var(--foreground);
          padding: 0.75rem 0;
          line-height: 1.5;
        }

        .message-input::placeholder {
          color: var(--muted-foreground);
        }

        .message-input:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .send-button {
          width: 36px;
          height: 36px;
          border-radius: 50%;
          border: none;
          background: linear-gradient(135deg, #3b82f6, #8b5cf6);
          color: white;
          cursor: pointer;
          display: flex;
          align-items: center;
          justify-content: center;
          transition: all 0.2s ease;
          box-shadow: 0 2px 8px rgba(59, 130, 246, 0.3);
        }

        .send-button:hover:not(.disabled) {
          transform: translateY(-1px);
          box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4);
        }

        .send-button:active:not(.disabled) {
          transform: translateY(0);
        }

        .send-button.disabled {
          background: var(--muted);
          color: var(--muted-foreground);
          cursor: not-allowed;
          box-shadow: none;
        }

        /* 滚动条样式 */
        .messages-list::-webkit-scrollbar {
          width: 6px;
        }

        .messages-list::-webkit-scrollbar-track {
          background: var(--muted);
          border-radius: 3px;
        }

        .messages-list::-webkit-scrollbar-thumb {
          background: var(--muted-foreground);
          border-radius: 3px;
        }

        .messages-list::-webkit-scrollbar-thumb:hover {
          background: var(--foreground);
        }

        /* 响应式设计 */
        @media (max-width: 768px) {
          .messages-list {
            padding: 1rem;
            gap: 1rem;
          }

          .input-container {
            padding: 1rem;
          }

          .message-bubble {
            max-width: 85%;
          }
        }
      `}</style>
    </>
  )
}

// 渲染应用
const container = document.getElementById('chat-root')
if (container) {
  const root = createRoot(container)
  root.render(<MAAChat />)
} else {
  console.error('找不到 chat-root 元素')
}