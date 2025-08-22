import React, { useState, useEffect, useRef } from 'react'
import { createRoot } from 'react-dom/client'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'

// 现代化的MAA聊天组件 - 基于 assistant-ui 设计风格
const MAAChat = () => {
  // 组件初始化日志只在首次加载时显示
  const initRef = useRef(false)
  const messageIdRef = useRef(0) // 消息ID计数器，确保唯一性
  if (!initRef.current) {
    console.log('🎬 MAA聊天组件初始化 (React', React.version, ')')
    initRef.current = true
  }
  
  // 生成唯一消息ID
  const generateMessageId = () => {
    messageIdRef.current += 1
    return `msg_${Date.now()}_${messageIdRef.current}`
  }
  
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
  const [sseConnected, setSseConnected] = useState(false)
  const [taskUpdates, setTaskUpdates] = useState({}) // 存储任务实时更新
  const messagesEndRef = useRef(null)
  const sseRef = useRef(null)

  // 处理重置按钮（需要先定义，才能在useEffect中调用）
  const handleReset = async () => {
    console.log('🔄 开始重置对话...')
    
    try {
      // 发送重置请求到后端
      const response = await fetch('http://localhost:8080/chat/reset', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        }
      })

      if (response.ok) {
        const data = await response.json()
        console.log('✅ 后端重置成功:', data)
        
        // 重置消息列表，使用后端返回的欢迎消息
        setMessages([{
          id: generateMessageId(),
          role: 'assistant',
          content: data.choices[0].message.content
        }])
        
        console.log('🎉 对话历史已清除')
      } else {
        throw new Error(`重置失败: ${response.status}`)
      }
    } catch (error) {
      console.error('❌ 重置失败:', error)
      
      // 即使后端重置失败，也清空前端消息列表
      setMessages([{
        id: generateMessageId(),
        role: 'assistant', 
        content: '对话已重置！我是MAA智能助手，可以帮您控制明日方舟自动化助手进行各种游戏操作。\n\n请问有什么可以为您效劳的吗？'
      }])
      
      console.log('⚠️ 使用默认消息重置对话')
    }
  }

  // 处理SSE事件的统一函数 - 显示所有事件
  const handleSSEEvent = (event) => {
    try {
      const data = JSON.parse(event.data)
      
      // 记录所有事件（除了心跳）
      if (event.type !== 'heartbeat') {
        console.log(`📨 收到${event.type}事件:`, data)
      }
      
      // 更新任务状态
      if (data.task_id) {
        setTaskUpdates(prev => ({
          ...prev,
          [data.task_id]: data
        }))
      }
      
      // 为所有事件类型添加通知消息
      let notificationMessage = ''
      let notificationIcon = ''
      
      switch (event.type) {
        case 'started':
          notificationIcon = '🚀'
          notificationMessage = `任务启动：${data.message || data.task_type}`
          break
        case 'progress':
          notificationIcon = '⏳'
          notificationMessage = `任务进行中：${data.message}`
          break
        case 'completed':
          notificationIcon = '✅'
          notificationMessage = `任务完成：${data.message}`
          break
        case 'failed':
          notificationIcon = '❌'
          notificationMessage = `任务失败：${data.message}`
          break
        case 'taskchain_started':
          notificationIcon = '🎬'
          notificationMessage = `任务链开始：${data.message}`
          break
        case 'taskchain_completed':
          notificationIcon = '🎉'
          notificationMessage = `任务链完成：${data.message}`
          break
        case 'subtask_started':
          notificationIcon = '🔧'
          notificationMessage = `子任务开始：${data.message}`
          break
        case 'subtask_completed':
          notificationIcon = '✅'
          notificationMessage = `子任务完成：${data.message}`
          break
        default:
          notificationIcon = '📨'
          notificationMessage = `${event.type}: ${data.message || JSON.stringify(data)}`
      }
      
      // 为所有任务事件添加聊天消息（除了心跳）
      if (event.type !== 'heartbeat' && notificationMessage) {
        setMessages(prev => [...prev, {
          id: generateMessageId(),
          role: 'assistant',
          content: `${notificationIcon} ${notificationMessage}`
        }])
      }
    } catch (error) {
      console.error(`❌ 解析${event.type}事件失败:`, error)
    }
  }

  // 处理心跳事件 - 静默处理，定期确认连接状态
  const handleHeartbeat = (event) => {
    try {
      const data = JSON.parse(event.data)
      
      // 只在特定时机输出心跳日志
      if (!window.heartbeatCount) window.heartbeatCount = 0
      window.heartbeatCount++
      
      // 每30次心跳输出一次日志（约5分钟）
      if (window.heartbeatCount % 30 === 1) {
        console.log('💓 SSE连接正常，心跳计数:', window.heartbeatCount)
      }
      
      // 不再自动显示心跳UI消息，避免刷屏
    } catch (error) {
      console.error('❌ 解析心跳事件失败:', error)
    }
  }

  // 连接SSE获取实时任务更新
  const connectSSE = () => {
    if (sseRef.current) {
      console.log('🔄 关闭现有SSE连接')
      sseRef.current.close()
    }

    console.log('🔗 正在连接SSE...')
    const eventSource = new EventSource('http://localhost:8080/sse/tasks')
    sseRef.current = eventSource

    // 连接打开事件
    eventSource.onopen = () => {
      console.log('✅ SSE连接已建立')
      setSseConnected(true)
      
      // 只在首次连接时显示UI提示
      if (!window.sseConnectedOnce) {
        window.sseConnectedOnce = true
        setMessages(prev => [...prev, {
          id: generateMessageId(),
          role: 'assistant',
          content: '🔗 SSE实时连接已建立，开始接收任务更新...'
        }])
      }
    }

    // 添加所有自定义事件类型的监听器
    const eventTypes = [
      'started', 'progress', 'completed', 'failed',
      'taskchain_started', 'taskchain_completed', 'taskchain_failed',
      'subtask_started', 'subtask_completed', 'subtask_failed', 'subtask_info',
      'test', 'demo', 'frontend_test'  // 测试事件类型
    ]
    
    eventTypes.forEach(eventType => {
      eventSource.addEventListener(eventType, handleSSEEvent)
    })
    console.log(`🎯 已注册${eventTypes.length}个SSE事件监听器`)

    // 心跳事件监听器
    eventSource.addEventListener('heartbeat', handleHeartbeat)

    // 默认消息处理器（处理没有指定类型的事件）
    eventSource.onmessage = (event) => {
      // 只记录非心跳的默认消息
      if (!event.data.includes('"message":"连接正常"')) {
        console.log('📨 收到默认消息事件:', event.data)
      }
      handleSSEEvent({...event, type: 'default'})
    }

    // 错误处理
    eventSource.onerror = (error) => {
      console.error('❌ SSE连接错误:', error)
      console.error('❌ SSE错误详情:', {
        readyState: eventSource.readyState,
        url: eventSource.url,
        withCredentials: eventSource.withCredentials
      })
      setSseConnected(false)
      
      // 只在首次错误时显示错误信息到UI
      if (!window.sseErrorShown) {
        window.sseErrorShown = true
        setMessages(prev => [...prev, {
          id: generateMessageId(),
          role: 'assistant',
          content: `❌ SSE连接中断，正在后台重试连接...`
        }])
      }
      
      // 5秒后重试连接
      setTimeout(() => {
        if (isConnected) {
          console.log('🔄 重新连接SSE...')
          connectSSE()
        }
      }, 5000)
    }
  }

  // 页面加载时重置对话和立即连接SSE
  useEffect(() => {
    console.log('🚀 初始化MAA聊天组件')
    handleReset()
    
    // 立即连接SSE，不等待MAA连接状态
    setIsConnected(true) // 设置为已连接，允许SSE连接
    connectSSE()
  }, [])

  // 管理SSE连接状态 - 仅在需要时重连
  useEffect(() => {
    // 只有在明确需要重连时才执行
    if (isConnected && !sseConnected && !sseRef.current) {
      const timer = setTimeout(() => {
        console.log('📡 重新建立SSE连接')
        connectSSE()
      }, 1000) // 延迟连接避免频繁重连
      
      return () => clearTimeout(timer)
    }
  }, [isConnected, sseConnected])
  
  // 组件卸载时清理SSE
  useEffect(() => {
    return () => {
      if (sseRef.current) {
        console.log('🧹 组件卸载，关闭SSE连接')
        sseRef.current.close()
        sseRef.current = null
      }
    }
  }, [])

  // 检查MAA连接 - 优化版本，减少日志输出
  useEffect(() => {
    let intervalRef = null
    
    const checkConnection = async () => {
      try {
        const response = await fetch('http://localhost:8080/health')
        
        if (response.ok) {
          const data = await response.json()
          
          // 检查MAA是否已经准备就绪
          const maaReady = data.status === 'ready' && 
                          data.maa_core && 
                          data.maa_core.connected === true
          
          if (maaReady && !isConnected) {
            console.log('🎉 MAA设备连接成功!')
            setIsConnected(true)
            updateStatus('connected')
          } else if (data.status === 'initializing' && isConnected) {
            console.log('🔄 MAA正在初始化设备连接...')
            setIsConnected(false)
            updateStatus('loading')
          } else if (!maaReady && isConnected) {
            console.log('⚠️ MAA设备连接中断')
            setIsConnected(false)
            updateStatus('disconnected')
          }
        } else if (isConnected) {
          console.log('❌ 后端响应错误:', response.status)
          setIsConnected(false)
          updateStatus('disconnected')
        }
      } catch (error) {
        if (isConnected) {
          console.error('🚨 连接检查失败:', error.message)
          setIsConnected(false)
          updateStatus('disconnected')
        }
      }
    }

    console.log('🚀 开始MAA服务器状态监控')
    checkConnection()
    intervalRef = setInterval(checkConnection, 10000) // 增加间隔到10秒
    
    return () => {
      if (intervalRef) {
        clearInterval(intervalRef)
      }
    }
  }, [])

  // 滚动到底部
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  // 更新状态
  const updateStatus = (status) => {
    console.log('🔄 更新状态:', status)
    
    const statusEl = document.getElementById('status')
    const infoEl = document.getElementById('info')
    
    console.log('🎯 DOM元素:', {
      statusEl: !!statusEl,
      infoEl: !!infoEl
    })
    
    if (statusEl && infoEl) {
      statusEl.className = `status ${status}`
      switch (status) {
        case 'connected':
          statusEl.innerHTML = `<div class="status-dot"></div>MAA设备已连接${sseConnected ? ' • SSE已连接' : ''}`
          infoEl.textContent = `可以开始对话${sseConnected ? ' • 支持实时更新' : ''}`
          console.log('🟢 状态设置为: 已连接')
          break
        case 'disconnected':
          statusEl.innerHTML = '<div class="status-dot"></div>MAA设备未连接'
          infoEl.textContent = '请检查设备连接或后端服务'
          console.log('🔴 状态设置为: 未连接')
          break
        case 'loading':
          statusEl.innerHTML = '<div class="status-dot"></div>正在初始化MAA...'
          infoEl.textContent = '正在连接设备，请稍候'
          console.log('🟡 状态设置为: 初始化中')
          break
      }
    } else {
      console.warn('⚠️ 无法找到状态DOM元素')
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
    // 过滤消息：排除截图消息，避免发送大量base64数据
    const filteredMessages = messages
      .filter(msg => {
        // 排除截图类型的消息
        if (msg.type === 'screenshot' || msg.type === 'screenshot_ref' || msg.type === 'screenshot_display') {
          console.log('🚫 过滤截图消息，避免发送base64数据')
          return false
        }
        
        // 排除包含大量数据的消息
        if (typeof msg.content === 'string' && msg.content.length > 10000) {
          console.log('🚫 过滤超长消息，长度:', msg.content.length)
          return false
        }
        
        // 排除包含base64的消息
        if (typeof msg.content === 'string' && 
            (msg.content.includes('base64') || msg.content.includes('data:image'))) {
          console.log('🚫 过滤包含图片数据的消息')
          return false
        }
        
        return true
      })
      .map(msg => ({
        role: msg.role,
        content: typeof msg.content === 'string' ? msg.content : 
                 typeof msg.content === 'object' ? msg.content.text || JSON.stringify(msg.content) :
                 String(msg.content)
      }))
      .slice(-8) // 只取最近8条消息
    
    console.log(`📤 发送消息给AI: ${filteredMessages.length} 条 (原始: ${messages.length} 条)`)
    
    const response = await fetch('http://localhost:8080/chat', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        messages: filteredMessages,
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
      id: generateMessageId(),
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
            id: generateMessageId(),
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
              // 特殊处理截图结果 - 兼容不同服务器格式
              if (functionName === 'maa_take_screenshot' && result.success && result.result) {
                let base64Data, fileSize, timestamp, screenshotId;
                
                if (result.result.screenshot) {
                  // 优化服务器格式
                  base64Data = result.result.screenshot;
                  fileSize = result.result.size || 0;
                  timestamp = result.result.timestamp;
                  screenshotId = `screenshot_${Date.now()}`;
                } else if (result.result.base64_data) {
                  // 智能服务器格式
                  base64Data = result.result.base64_data;
                  fileSize = result.result.file_size || 0;
                  timestamp = result.result.timestamp;
                  screenshotId = result.result.screenshot_id || `screenshot_${Date.now()}`;
                }
                
                if (base64Data) {
                  // 创建截图消息，使用base64 URL但不保存到历史
                  const screenshotUrl = `data:image/png;base64,${base64Data}`;
                  
                  setMessages(prev => [...prev, {
                    id: generateMessageId(),
                    role: 'assistant',
                    type: 'screenshot_display',
                    content: {
                      text: `✅ 截图完成！\n\n*这是MAA当前看到的游戏画面*\n\n**截图信息:**\n- 数据大小: ${Math.round(fileSize / 1024)}KB\n- 时间戳: ${timestamp ? new Date(timestamp).toLocaleString() : '未知'}\n- 服务器: ${result.backend || 'MAA'}\n\n点击图片可放大查看`,
                      screenshotUrl: screenshotUrl,
                      originalUrl: screenshotUrl,
                      screenshotId: screenshotId
                    }
                  }]);
                  
                  // 跳过后面的普通结果显示逻辑
                  resultText = null;
                }
              } else {
                resultText = `✅ 执行成功\n\n\`\`\`json\n${JSON.stringify(result.result, null, 2)}\n\`\`\``
              }
            }
          }
          
          // 只有非截图结果才显示普通的结果文本
          if (resultText) {
            setMessages(prev => [...prev, {
              id: generateMessageId(),
              role: 'assistant',
              content: resultText
            }])
          }
          
          // AI的额外回复
          if (choice.message.content) {
            setMessages(prev => [...prev, {
              id: generateMessageId(),
              role: 'assistant',
              content: choice.message.content
            }])
          }
        } else {
          // 直接回复
          setMessages(prev => [...prev, {
            id: generateMessageId(),
            role: 'assistant',
            content: choice.message.content || '我理解了您的需求，但暂时无法执行相关操作。如果您需要执行MAA操作，请尝试更具体的指令。'
          }])
        }
      } else {
        setMessages(prev => [...prev, {
          id: generateMessageId(),
          role: 'assistant',
          content: '抱歉，我没有理解您的意思。您可以尝试说：\n• "帮我截个图"\n• "获取MAA状态"\n• "查看我的干员"'
        }])
      }
    } catch (error) {
      console.error('处理消息失败:', error)
      setMessages(prev => [...prev, {
        id: generateMessageId(),
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

  // 处理截图按钮
  const handleScreenshot = async () => {
    if (!isConnected || isLoading) return

    setIsLoading(true)
    updateStatus('loading')

    try {
      // 调用MAA截图工具
      const response = await callMAAFunction('maa_take_screenshot', {})
      
      console.log('📸 截图响应数据:', response)
      
      if (response.success && response.result) {
        // 兼容不同的响应格式
        let base64Data, fileSize, timestamp, screenshotId;
        
        if (response.result.screenshot) {
          // 优化服务器格式
          base64Data = response.result.screenshot;
          fileSize = response.result.size || 0;
          timestamp = response.result.timestamp;
          screenshotId = `screenshot_${Date.now()}`;
        } else if (response.result.base64_data) {
          // 智能服务器格式
          base64Data = response.result.base64_data;
          fileSize = response.result.file_size || 0;
          timestamp = response.result.timestamp;
          screenshotId = response.result.screenshot_id || `screenshot_${Date.now()}`;
        } else {
          throw new Error('未找到截图数据');
        }
        
        console.log('✅ 截图base64数据长度:', base64Data?.length)
        
        // 验证base64数据
        if (!base64Data || base64Data.length === 0) {
          throw new Error('截图数据为空');
        }
        
        // 创建截图消息，临时显示但不发送给AI
        const screenshotUrl = `data:image/png;base64,${base64Data}`;
        
        // 调试信息
        console.log('📸 Screenshot Debug Info:');
        console.log('- Base64 length:', base64Data.length);
        console.log('- File size:', fileSize, 'bytes');
        console.log('- Data URL length:', screenshotUrl.length);
        console.log('- Base64 starts with:', base64Data.substring(0, 50));
        
        // 检查浏览器的data URL限制
        if (screenshotUrl.length > 2000000) { // 2MB limit for Chrome
          console.warn('⚠️ Data URL might exceed browser limits:', screenshotUrl.length, 'characters');
        }
        
        setMessages(prev => [...prev, {
          id: generateMessageId(),
          role: 'assistant',
          type: 'screenshot_display',
          content: {
            text: `📸 截图完成！\n\n*这是MAA当前看到的游戏画面*\n\n**截图信息:**\n- 数据大小: ${Math.round(fileSize / 1024)}KB (Base64: ${Math.round(base64Data.length / 1024)}KB)\n- 时间戳: ${timestamp ? new Date(timestamp).toLocaleString() : '未知'}\n- 服务器: ${response.backend || 'MAA'}\n- Data URL长度: ${Math.round(screenshotUrl.length / 1024)}KB\n\n点击图片可放大查看`,
            screenshotUrl: screenshotUrl,
            originalUrl: screenshotUrl, // 使用相同的URL
            screenshotId: screenshotId,
            debugInfo: {
              base64Length: base64Data.length,
              fileSize: fileSize,
              dataUrlLength: screenshotUrl.length
            }
          }
        }])
      } else {
        console.error('❌ 截图失败响应:', response)
        throw new Error(response.error?.message || response.message || '截图请求失败')
      }
    } catch (error) {
      console.error('❌ 截图失败:', error)
      setMessages(prev => [...prev, {
        id: generateMessageId(),
        role: 'assistant',
        content: `❌ 截图失败：${error.message}`
      }])
    } finally {
      setIsLoading(false)
      updateStatus(isConnected ? 'connected' : 'disconnected')
    }
  }


  return (
    <>
      <div className="chat-interface">
        {/* 消息列表 */}
        <div className="messages-container">
          <div className="messages-list">
            {messages.slice(-20).map((message) => (  /* 只显示最近20条消息以提升性能 */
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
                    {(message.type === 'screenshot' || message.type === 'screenshot_display') ? (
                      <div className="screenshot-message">
                        <ReactMarkdown 
                          remarkPlugins={[remarkGfm]}
                          skipHtml={true}
                        >
                          {message.content.text}
                        </ReactMarkdown>
                        <div className="screenshot-container">
                          <img 
                            src={message.content.screenshotUrl || `data:image/jpeg;base64,${message.content.thumbnailBase64}`}
                            alt="MAA截图"
                            className="screenshot-thumbnail"
                            onClick={() => window.open(message.content.originalUrl, '_blank')}
                            title="点击查看原图"
                            onLoad={(e) => {
                              console.log('✅ Image loaded successfully:', e.target.naturalWidth, 'x', e.target.naturalHeight);
                            }}
                            onError={(e) => {
                              console.error('❌ Image failed to load:', e);
                              e.target.style.display = 'none';
                              // 显示错误信息
                              const errorDiv = document.createElement('div');
                              errorDiv.textContent = '图片加载失败 - 点击查看调试信息';
                              errorDiv.style.cssText = 'padding: 20px; background: #f0f0f0; border: 1px dashed #ccc; text-align: center; cursor: pointer;';
                              errorDiv.onclick = () => {
                                console.log('Debug info:', message.content.debugInfo);
                              };
                              e.target.parentNode.insertBefore(errorDiv, e.target.nextSibling);
                            }}
                          />
                          <div className="screenshot-overlay">
                            <span>点击查看原图</span>
                          </div>
                        </div>
                      </div>
                    ) : (
                      <ReactMarkdown 
                        remarkPlugins={[remarkGfm]}
                        skipHtml={true}
                      >
                        {message.content}
                      </ReactMarkdown>
                    )}
                  </div>
                </div>
              </div>
            ))}
            <div ref={messagesEndRef} />
          </div>
        </div>
        
        {/* 工具栏 */}
        <div className="toolbar">
          <button
            onClick={handleScreenshot}
            disabled={!isConnected || isLoading}
            className={`tool-button screenshot-button ${(!isConnected || isLoading) ? 'disabled' : ''}`}
            title="截图"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M14.828 14.828a4 4 0 0 1-5.656 0"></path>
              <path d="M9 9a3 3 0 1 1 6 0c0 .833-.333 1.5-1 2s-1.5.5-2 .5-.333-.167-1-.5-1-1.167-1-2z"></path>
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
              <path d="M21 10a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v1"></path>
              <path d="M7 8V6a2 2 0 0 1 2-2h6a2 2 0 0 1 2 2v2"></path>
            </svg>
            截图
          </button>
          <button
            onClick={handleReset}
            className="tool-button reset-button"
            title="重置对话"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polyline points="23,4 23,10 17,10"></polyline>
              <polyline points="1,20 1,14 7,14"></polyline>
              <path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 0 1 3.51 15"></path>
            </svg>
            重置
          </button>
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

        /* Markdown 样式 */
        .message-bubble h1, .message-bubble h2, .message-bubble h3 {
          margin: 0.5rem 0;
          font-weight: 600;
        }

        .message-bubble p {
          margin: 0.25rem 0;
        }

        .message-bubble pre {
          background: rgba(0, 0, 0, 0.05);
          padding: 0.5rem;
          border-radius: 0.375rem;
          overflow-x: auto;
          margin: 0.5rem 0;
        }

        .message-bubble code {
          background: rgba(0, 0, 0, 0.05);
          padding: 0.125rem 0.25rem;
          border-radius: 0.25rem;
          font-size: 0.875em;
        }

        .message-bubble pre code {
          background: none;
          padding: 0;
        }

        .message-bubble img {
          max-width: 100%;
          height: auto;
          border-radius: 0.5rem;
          margin: 0.5rem 0;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        .message-bubble ul, .message-bubble ol {
          margin: 0.5rem 0;
          padding-left: 1.5rem;
        }

        .message-bubble li {
          margin: 0.25rem 0;
        }

        .message-bubble blockquote {
          border-left: 3px solid var(--border);
          padding-left: 1rem;
          margin: 0.5rem 0;
          font-style: italic;
          opacity: 0.8;
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

        /* 工具栏样式 */
        .toolbar {
          padding: 1rem 1.5rem 0;
          display: flex;
          gap: 0.75rem;
          justify-content: center;
          border-top: 1px solid var(--border);
          background: var(--background);
        }

        .tool-button {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.5rem 1rem;
          border: 1px solid var(--border);
          border-radius: 0.75rem;
          background: var(--card);
          color: var(--card-foreground);
          font-size: 0.875rem;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s ease;
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }

        .tool-button:hover:not(.disabled) {
          background: var(--accent);
          color: var(--accent-foreground);
          transform: translateY(-1px);
          box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
        }

        .tool-button:active:not(.disabled) {
          transform: translateY(0);
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }

        .tool-button.disabled {
          opacity: 0.5;
          cursor: not-allowed;
          background: var(--muted);
          color: var(--muted-foreground);
        }

        .screenshot-button {
          background: linear-gradient(135deg, #10b981, #059669);
          color: white;
          border: none;
        }

        .screenshot-button:hover:not(.disabled) {
          background: linear-gradient(135deg, #059669, #047857);
          color: white;
        }

        .reset-button {
          background: linear-gradient(135deg, #f59e0b, #d97706);
          color: white;
          border: none;
        }

        .reset-button:hover:not(.disabled) {
          background: linear-gradient(135deg, #d97706, #b45309);
          color: white;
        }

        /* 截图样式 */
        .screenshot-message {
          display: flex;
          flex-direction: column;
          gap: 0.75rem;
        }

        .screenshot-container {
          position: relative;
          display: inline-block;
          border-radius: 0.5rem;
          overflow: hidden;
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
          cursor: pointer;
          transition: all 0.2s ease;
          max-width: 100%;
        }

        .screenshot-container:hover {
          transform: translateY(-2px);
          box-shadow: 0 6px 20px rgba(0, 0, 0, 0.2);
        }

        .screenshot-thumbnail {
          max-width: 100%;
          height: auto;
          display: block;
          border-radius: 0.5rem;
          max-height: 400px;
          object-fit: contain;
        }

        .screenshot-overlay {
          position: absolute;
          bottom: 0;
          left: 0;
          right: 0;
          background: linear-gradient(to top, rgba(0, 0, 0, 0.8), transparent);
          color: white;
          padding: 0.5rem;
          text-align: center;
          font-size: 0.875rem;
          opacity: 0;
          transition: opacity 0.2s ease;
        }

        .screenshot-container:hover .screenshot-overlay {
          opacity: 1;
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
  console.log('🌟 渲染MAA聊天应用')
  const root = createRoot(container)
  root.render(<MAAChat />)
} else {
  console.error('❌ 找不到 chat-root 元素')
}