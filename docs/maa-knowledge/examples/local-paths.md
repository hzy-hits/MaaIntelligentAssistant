# MAA 本地路径探测结果

## MaaCore 动态库位置

### 主要库文件 (macOS)
**位置**: `/Users/ivena/Library/Application Support/com.loong.maa/lib/`

```
libMaaCore.dylib                    # MAA核心库
libfastdeploy_ppocr.dylib          # OCR文字识别库
libonnxruntime.1.18.0.dylib       # ONNX运行时库
libonnxruntime.dylib               # ONNX运行时库（符号链接）
libopencv_world4.408.dylib         # OpenCV计算机视觉库
libopencv_world4.dylib             # OpenCV库（符号链接）
```

### 关键发现
- **libMaaCore.dylib** 是主要的动态库文件，我们的Rust项目需要链接这个库
- 所有依赖库都在同一目录，包括OCR、ML推理、图像处理等功能
- 路径是maa-cli的标准安装位置

## MAA 资源文件结构

### 核心配置文件
**位置**: `/Users/ivena/Library/Application Support/com.loong.maa/`

```
MaaResource/                       # 官方资源文件
├── LICENSE
├── README.md
├── cache/
├── resource/
│   ├── battle_data.json          # 战斗数据
│   ├── infrast.json              # 基建数据
│   ├── item_index.json           # 物品索引
│   ├── recruitment.json          # 公招数据
│   ├── stages.json               # 关卡数据
│   └── version.json              # 版本信息

resource/                          # 用户资源目录
├── Arknights-Tile-Pos/           # 关卡地图数据（数千个JSON文件）
├── battle_data.json
├── config.json
├── infrast.json
├── item_index.json
├── ocr_config.json
├── recruitment.json
├── stages.json
└── version.json
```

### 配置和缓存
```
config/
└── profiles/
    └── default.json              # 默认配置文件

cache/
└── avatars/                      # 头像缓存

debug/
└── asst.log                      # 调试日志
```

## 系统目录映射

### MAA-CLI 目录结构
```bash
数据根目录:    /Users/ivena/Library/Application Support/com.loong.maa
配置目录:      /Users/ivena/Library/Application Support/com.loong.maa/config
缓存目录:      /Users/ivena/Library/Caches/com.loong.maa
库文件目录:    /Users/ivena/Library/Application Support/com.loong.maa/lib
日志目录:      /Users/ivena/Library/Application Support/com.loong.maa/debug
热更新目录:    /Users/ivena/Library/Application Support/com.loong.maa/MaaResource
资源目录:      /Users/ivena/Library/Application Support/com.loong.maa/resource
```

## 集成路径配置

### 对于Rust项目的重要路径

1. **动态库链接路径**:
   ```rust
   // 在 Cargo.toml 或构建脚本中需要配置
   "/Users/ivena/Library/Application Support/com.loong.maa/lib/libMaaCore.dylib"
   ```

2. **资源文件路径**:
   ```rust
   // MAA资源目录，运行时需要
   "/Users/ivena/Library/Application Support/com.loong.maa/resource"
   ```

3. **配置文件路径**:
   ```rust
   // 用户配置文件
   "/Users/ivena/Library/Application Support/com.loong.maa/config"
   ```

## 环境变量建议

```bash
# 可以设置的环境变量
export MAA_CORE_DIR="/Users/ivena/Library/Application Support/com.loong.maa"
export MAA_LIB_PATH="/Users/ivena/Library/Application Support/com.loong.maa/lib"
export MAA_RESOURCE_PATH="/Users/ivena/Library/Application Support/com.loong.maa/resource"
```

## 集成验证

### 文件存在性检查
- ✅ `libMaaCore.dylib` 存在且可访问
- ✅ 资源文件完整（包含所有关键JSON文件）
- ✅ 配置目录结构正确
- ✅ maa-cli 可以正常运行

### 下一步集成计划
1. 在Rust项目中配置动态库路径
2. 使用 `maa-sys` crate 进行FFI调用
3. 验证库文件加载和基本函数调用
4. 测试MAA功能完整性

## 注意事项

- macOS系统路径包含空格，在构建脚本中需要正确处理
- 动态库依赖关系完整，无需额外安装依赖
- maa-cli v0.5.4 与 MaaCore v5.22.3 版本匹配
- 所有文件权限正常，可以被当前用户访问