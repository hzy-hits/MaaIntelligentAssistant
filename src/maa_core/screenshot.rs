//! MAA 截图功能模块
//! 
//! 提供MAA截图保存、访问和管理功能

use std::path::PathBuf;
use std::fs;
use chrono::{DateTime, Utc};
use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use tracing::{info, warn};
use image::{ImageFormat, DynamicImage, imageops::FilterType};

/// 截图信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScreenshotInfo {
    /// 截图ID
    pub id: String,
    /// 截图时间
    pub timestamp: DateTime<Utc>,
    /// 原始文件路径
    pub file_path: String,
    /// 缩略图文件路径
    pub thumbnail_path: String,
    /// 原始文件大小（字节）
    pub file_size: u64,
    /// 缩略图文件大小（字节）
    pub thumbnail_size: u64,
    /// 图片尺寸
    pub dimensions: Option<(u32, u32)>,
    /// 缩略图尺寸
    pub thumbnail_dimensions: Option<(u32, u32)>,
    /// Base64编码的缩略图数据（用于直接返回）
    pub thumbnail_base64: Option<String>,
}

/// 截图管理器
pub struct ScreenshotManager {
    /// 截图保存目录
    screenshots_dir: PathBuf,
}

impl ScreenshotManager {
    /// 创建截图管理器
    pub fn new() -> Result<Self> {
        let screenshots_dir = PathBuf::from("screenshots");
        
        // 确保截图目录存在
        if !screenshots_dir.exists() {
            fs::create_dir_all(&screenshots_dir)
                .map_err(|e| anyhow!("创建截图目录失败: {}", e))?;
        }
        
        Ok(Self { screenshots_dir })
    }
    
    /// 保存截图数据并返回截图信息（包含压缩的缩略图）
    pub fn save_screenshot(&self, image_data: Vec<u8>) -> Result<ScreenshotInfo> {
        let timestamp = Utc::now();
        let id = format!("screenshot_{}", timestamp.format("%Y%m%d_%H%M%S_%3f"));
        let filename = format!("{}.png", id);
        let thumbnail_filename = format!("{}_thumb.jpg", id);
        let file_path = self.screenshots_dir.join(&filename);
        let thumbnail_path = self.screenshots_dir.join(&thumbnail_filename);
        
        // 1. 保存原始截图文件
        fs::write(&file_path, &image_data)
            .map_err(|e| anyhow!("保存截图失败: {}", e))?;
        
        // 2. 加载图片并获取尺寸
        let img = image::load_from_memory(&image_data)
            .map_err(|e| anyhow!("解析图片失败: {}", e))?;
        
        let (width, height) = (img.width(), img.height());
        
        // 3. 生成缩略图 (最大宽度800px，保持比例)
        let thumbnail = self.create_thumbnail(&img, 800)?;
        let (thumb_width, thumb_height) = (thumbnail.width(), thumbnail.height());
        
        // 4. 保存缩略图为JPEG格式（更小的文件）
        let mut thumb_data = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut thumb_data);
        thumbnail.write_to(&mut cursor, ImageFormat::Jpeg)
            .map_err(|e| anyhow!("保存缩略图失败: {}", e))?;
        
        fs::write(&thumbnail_path, &thumb_data)
            .map_err(|e| anyhow!("写入缩略图文件失败: {}", e))?;
        
        // 5. 生成缩略图的Base64数据用于直接返回
        let thumbnail_base64 = BASE64.encode(&thumb_data);
        
        let file_size = image_data.len() as u64;
        let thumbnail_size = thumb_data.len() as u64;
        
        let screenshot = ScreenshotInfo {
            id: id.clone(),
            timestamp,
            file_path: file_path.to_string_lossy().to_string(),
            thumbnail_path: thumbnail_path.to_string_lossy().to_string(),
            file_size,
            thumbnail_size,
            dimensions: Some((width, height)),
            thumbnail_dimensions: Some((thumb_width, thumb_height)),
            thumbnail_base64: Some(thumbnail_base64),
        };
        
        info!("截图已保存: {} (原图: {}KB, 缩略图: {}KB)", 
              screenshot.id, file_size / 1024, thumbnail_size / 1024);
        Ok(screenshot)
    }
    
    /// 创建缩略图
    fn create_thumbnail(&self, img: &DynamicImage, max_width: u32) -> Result<DynamicImage> {
        let (width, height) = (img.width(), img.height());
        
        // 如果图片宽度小于等于最大宽度，直接返回
        if width <= max_width {
            return Ok(img.clone());
        }
        
        // 计算保持比例的新尺寸
        let ratio = max_width as f32 / width as f32;
        let new_height = (height as f32 * ratio) as u32;
        
        // 使用高质量的缩放算法
        let thumbnail = img.resize(max_width, new_height, FilterType::Lanczos3);
        Ok(thumbnail)
    }
    
    /// 根据ID获取截图
    pub fn get_screenshot(&self, id: &str) -> Result<ScreenshotInfo> {
        let filename = format!("{}.png", id);
        let thumbnail_filename = format!("{}_thumb.jpg", id);
        let file_path = self.screenshots_dir.join(&filename);
        let thumbnail_path = self.screenshots_dir.join(&thumbnail_filename);
        
        if !file_path.exists() {
            return Err(anyhow!("截图不存在: {}", id));
        }
        
        // 读取原始文件数据
        let image_data = fs::read(&file_path)
            .map_err(|e| anyhow!("读取截图失败: {}", e))?;
        let file_size = image_data.len() as u64;
        
        // 读取缩略图数据（如果存在）
        let (thumbnail_base64, thumbnail_size) = if thumbnail_path.exists() {
            let thumb_data = fs::read(&thumbnail_path)
                .map_err(|e| anyhow!("读取缩略图失败: {}", e))?;
            let thumbnail_base64 = BASE64.encode(&thumb_data);
            let thumbnail_size = thumb_data.len() as u64;
            (Some(thumbnail_base64), thumbnail_size)
        } else {
            // 如果缩略图不存在，动态生成
            let img = image::load_from_memory(&image_data)
                .map_err(|e| anyhow!("解析图片失败: {}", e))?;
            let thumbnail = self.create_thumbnail(&img, 800)?;
            
            let mut thumb_data = Vec::new();
            let mut cursor = std::io::Cursor::new(&mut thumb_data);
            thumbnail.write_to(&mut cursor, ImageFormat::Jpeg)
                .map_err(|e| anyhow!("生成缩略图失败: {}", e))?;
            
            let thumbnail_base64 = BASE64.encode(&thumb_data);
            let thumbnail_size = thumb_data.len() as u64;
            (Some(thumbnail_base64), thumbnail_size)
        };
        
        // 从文件名解析时间戳
        let timestamp = self.parse_timestamp_from_id(id).unwrap_or_else(|| Utc::now());
        
        // 获取图片尺寸
        let img = image::load_from_memory(&image_data)
            .map_err(|e| anyhow!("解析图片失败: {}", e))?;
        let (width, height) = (img.width(), img.height());
        
        Ok(ScreenshotInfo {
            id: id.to_string(),
            timestamp,
            file_path: file_path.to_string_lossy().to_string(),
            thumbnail_path: thumbnail_path.to_string_lossy().to_string(),
            file_size,
            thumbnail_size,
            dimensions: Some((width, height)),
            thumbnail_dimensions: None, // 为了性能，不重新计算
            thumbnail_base64,
        })
    }
    
    /// 获取所有截图列表
    pub fn list_screenshots(&self) -> Result<Vec<ScreenshotInfo>> {
        let mut screenshots = Vec::new();
        
        let entries = fs::read_dir(&self.screenshots_dir)
            .map_err(|e| anyhow!("读取截图目录失败: {}", e))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| anyhow!("读取目录项失败: {}", e))?;
            let path = entry.path();
            
            if path.extension() == Some(std::ffi::OsStr::new("png")) {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem.starts_with("screenshot_") {
                        // 获取文件信息但不加载Base64数据（为了性能）
                        let metadata = entry.metadata()
                            .map_err(|e| anyhow!("获取文件信息失败: {}", e))?;
                        
                        let timestamp = self.parse_timestamp_from_id(stem)
                            .unwrap_or_else(|| {
                                metadata.modified()
                                    .map(|t| DateTime::from(t))
                                    .unwrap_or_else(|_| Utc::now())
                            });
                        
                        // 构建缩略图路径
                        let thumbnail_path = self.screenshots_dir.join(format!("{}_thumb.jpg", stem));
                        
                        screenshots.push(ScreenshotInfo {
                            id: stem.to_string(),
                            timestamp,
                            file_path: path.to_string_lossy().to_string(),
                            thumbnail_path: thumbnail_path.to_string_lossy().to_string(),
                            file_size: metadata.len(),
                            thumbnail_size: 0, // 列表时不计算缩略图大小
                            dimensions: None,
                            thumbnail_dimensions: None,
                            thumbnail_base64: None, // 列表时不加载Base64数据
                        });
                    }
                }
            }
        }
        
        // 按时间戳倒序排列
        screenshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(screenshots)
    }
    
    /// 清理旧截图（保留最近的N个）
    pub fn cleanup_old_screenshots(&self, keep_count: usize) -> Result<usize> {
        let mut screenshots = self.list_screenshots()?;
        
        if screenshots.len() <= keep_count {
            return Ok(0);
        }
        
        // 删除多余的截图
        let to_delete = screenshots.split_off(keep_count);
        let mut deleted_count = 0;
        
        for screenshot in to_delete {
            match fs::remove_file(&screenshot.file_path) {
                Ok(_) => {
                    deleted_count += 1;
                    info!("已删除旧截图: {}", screenshot.id);
                },
                Err(e) => {
                    warn!("删除截图失败: {} - {}", screenshot.id, e);
                }
            }
        }
        
        Ok(deleted_count)
    }
    
    /// 从ID解析时间戳
    fn parse_timestamp_from_id(&self, id: &str) -> Option<DateTime<Utc>> {
        // screenshot_20241219_143022_123
        if let Some(time_part) = id.strip_prefix("screenshot_") {
            let parts: Vec<&str> = time_part.split('_').collect();
            if parts.len() >= 3 {
                let date_part = parts[0]; // 20241219
                let time_part = parts[1]; // 143022
                let _ms_part = parts.get(2).unwrap_or(&"000"); // 123
                
                if date_part.len() == 8 && time_part.len() == 6 {
                    let datetime_str = format!("{}T{}Z", 
                        format!("{}-{}-{}", &date_part[0..4], &date_part[4..6], &date_part[6..8]),
                        format!("{}:{}:{}", &time_part[0..2], &time_part[2..4], &time_part[4..6])
                    );
                    
                    return DateTime::parse_from_rfc3339(&datetime_str).ok().map(|dt| dt.with_timezone(&Utc));
                }
            }
        }
        None
    }
}

/// 全局截图管理器实例
static SCREENSHOT_MANAGER: once_cell::sync::Lazy<ScreenshotManager> = 
    once_cell::sync::Lazy::new(|| {
        ScreenshotManager::new().expect("初始化截图管理器失败")
    });

/// 保存MAA截图并返回截图信息
pub fn save_maa_screenshot(image_data: Vec<u8>) -> Result<ScreenshotInfo> {
    SCREENSHOT_MANAGER.save_screenshot(image_data)
}

/// 获取截图
pub fn get_screenshot_by_id(id: &str) -> Result<ScreenshotInfo> {
    SCREENSHOT_MANAGER.get_screenshot(id)
}

/// 获取所有截图列表
pub fn list_all_screenshots() -> Result<Vec<ScreenshotInfo>> {
    SCREENSHOT_MANAGER.list_screenshots()
}

/// 清理旧截图
pub fn cleanup_screenshots(keep_count: usize) -> Result<usize> {
    SCREENSHOT_MANAGER.cleanup_old_screenshots(keep_count)
}