//! MAA FFI 绑定
//! 
//! MAA 官方 FFI 绑定的安全封装，提供线程安全的接口

use std::collections::HashMap;
use std::ffi::{c_void, NulError};
#[cfg(feature = "with-maa-core")]
use std::ffi::CString;

/// MAA 错误类型
#[derive(Debug)]
pub enum MaaError {
    Unknown,
    TooLargeAlloc,
    Null,
    Utf8,
}

impl From<NulError> for MaaError {
    fn from(_: NulError) -> Self {
        Self::Null
    }
}

impl From<std::str::Utf8Error> for MaaError {
    fn from(_: std::str::Utf8Error) -> Self {
        Self::Utf8
    }
}

/// 任务定义
#[derive(Debug, Clone)]
pub struct Task {
    pub id: i32,
    pub type_: String,
    pub params: String,
}

// 重新导出 MAA 官方绑定的类型
// 这些是从 maa-official/src/Rust/src/maa_sys/bind.rs 复制的基本类型定义

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AsstExtAPI {
    _unused: [u8; 0],
}

pub type AsstHandle = *mut AsstExtAPI;
pub type AsstBool = u8;
pub type AsstSize = u64;
pub type AsstMsgId = i32;
pub type AsstTaskId = i32;
pub type AsstAsyncCallId = i32;
pub type AsstStaticOptionKey = i32;
pub type AsstInstanceOptionKey = i32;
pub type AsstApiCallback = ::std::option::Option<
    unsafe extern "C" fn(
        msg: AsstMsgId,
        detail_json: *const ::std::os::raw::c_char,
        custom_arg: *mut ::std::os::raw::c_void,
    ),
>;

// MAA 核心 FFI 函数声明（这些需要链接到实际的 MAA 库）
extern "C" {
    pub fn AsstCreate() -> AsstHandle;
    pub fn AsstCreateEx(
        callback: AsstApiCallback,
        custom_arg: *mut ::std::os::raw::c_void,
    ) -> AsstHandle;
    pub fn AsstDestroy(handle: AsstHandle);
    pub fn AsstLoadResource(path: *const ::std::os::raw::c_char) -> AsstBool;
    pub fn AsstAsyncConnect(
        handle: AsstHandle,
        adb_path: *const ::std::os::raw::c_char,
        address: *const ::std::os::raw::c_char,
        config: *const ::std::os::raw::c_char,
        block: AsstBool,
    ) -> AsstAsyncCallId;
    pub fn AsstStart(handle: AsstHandle) -> AsstBool;
    pub fn AsstStop(handle: AsstHandle) -> AsstBool;
    pub fn AsstRunning(handle: AsstHandle) -> AsstBool;
    pub fn AsstAsyncClick(handle: AsstHandle, x: i32, y: i32, block: AsstBool) -> AsstAsyncCallId;
    pub fn AsstGetImage(
        handle: AsstHandle,
        buff: *mut c_void,
        buff_size: AsstSize,
    ) -> AsstSize;
    pub fn AsstAppendTask(
        handle: AsstHandle,
        type_: *const ::std::os::raw::c_char,
        params: *const ::std::os::raw::c_char,
    ) -> AsstTaskId;
    pub fn AsstGetUUID(
        handle: AsstHandle,
        buff: *mut ::std::os::raw::c_char,
        buff_size: AsstSize,
    ) -> AsstSize;
    pub fn AsstGetNullSize() -> AsstSize;
}

/// 安全的 MAA 包装器
#[derive(Debug)]
pub struct SafeMaaWrapper {
    handle: AsstHandle,
    target: Option<String>,
    tasks: HashMap<i32, Task>,
}

impl SafeMaaWrapper {
    pub fn new() -> Self {
        #[cfg(feature = "with-maa-core")]
        unsafe {
            Self {
                handle: AsstCreate(),
                target: None,
                tasks: HashMap::new(),
            }
        }
        
        #[cfg(not(feature = "with-maa-core"))]
        {
            Self {
                handle: std::ptr::null_mut(),
                target: None,
                tasks: HashMap::new(),
            }
        }
    }

    pub fn with_callback_and_custom_arg(
        _callback: AsstApiCallback,
        _custom_arg: *mut c_void,
    ) -> Self {
        #[cfg(feature = "with-maa-core")]
        unsafe {
            Self {
                handle: AsstCreateEx(callback, custom_arg),
                target: None,
                tasks: HashMap::new(),
            }
        }
        
        #[cfg(not(feature = "with-maa-core"))]
        {
            Self {
                handle: std::ptr::null_mut(),
                target: None,
                tasks: HashMap::new(),
            }
        }
    }

    pub fn connect_safe(
        &mut self,
        _adb_path: &str,
        address: &str,
        _config: Option<&str>,
    ) -> Result<i32, MaaError> {
        #[cfg(feature = "with-maa-core")]
        {
            // 预先分配 CString 以避免 dangling pointer
            let c_adb_path = CString::new(adb_path)?;
            let c_address = CString::new(address)?;
            let c_config = config.map(|cfg| CString::new(cfg)).transpose()?;
            
            unsafe {
                let c_cfg_ptr = match c_config.as_ref() {
                    Some(cfg) => cfg.as_ptr(),
                    None => std::ptr::null(),
                };
                
                let ret = AsstAsyncConnect(
                    self.handle,
                    c_adb_path.as_ptr(),
                    c_address.as_ptr(),
                    c_cfg_ptr,
                    1,
                );
                
                if ret != 0 {
                    self.target = Some(address.to_string());
                    Ok(ret)
                } else {
                    Err(MaaError::Unknown)
                }
            }
        }
        
        #[cfg(not(feature = "with-maa-core"))]
        {
            self.target = Some(address.to_string());
            Ok(1) // 模拟成功
        }
    }

    pub fn screenshot(&self) -> Result<Vec<u8>, MaaError> {
        #[cfg(feature = "with-maa-core")]
        {
            unsafe {
                let mut buff_size = 2 * 1920 * 1080 * 4;
                loop {
                    if buff_size > 10 * 1920 * 1080 * 4 {
                        return Err(MaaError::TooLargeAlloc);
                    }
                    let mut buff: Vec<u8> = Vec::with_capacity(buff_size);
                    let data_size = AsstGetImage(
                        self.handle,
                        buff.as_mut_ptr() as *mut c_void,
                        buff_size as u64,
                    );
                    if data_size == Self::get_null_size() {
                        buff_size = 2 * buff_size;
                        continue;
                    }
                    buff.set_len(data_size as usize);
                    buff.resize(data_size as usize, 0);
                    return Ok(buff);
                }
            }
        }
        
        #[cfg(not(feature = "with-maa-core"))]
        {
            // 返回模拟的截图数据
            Ok(vec![0; 1920 * 1080 * 4])
        }
    }

    pub fn click(&self, _x: i32, _y: i32) -> Result<i32, MaaError> {
        #[cfg(feature = "with-maa-core")]
        {
            unsafe {
                let ret = AsstAsyncClick(self.handle, x, y, 0);
                if ret != 0 {
                    Ok(ret)
                } else {
                    Err(MaaError::Unknown)
                }
            }
        }
        
        #[cfg(not(feature = "with-maa-core"))]
        {
            Ok(1) // 模拟成功
        }
    }

    pub fn create_task(&mut self, type_: &str, params: &str) -> Result<AsstTaskId, MaaError> {
        #[cfg(feature = "with-maa-core")]
        {
            unsafe {
                let c_type = CString::new(type_)?;
                let c_params = CString::new(params)?;
                let task_id = AsstAppendTask(self.handle, c_type.as_ptr(), c_params.as_ptr());
                self.tasks.insert(
                    task_id,
                    Task {
                        id: task_id,
                        type_: type_.to_string(),
                        params: params.to_string(),
                    },
                );
                Ok(task_id)
            }
        }
        
        #[cfg(not(feature = "with-maa-core"))]
        {
            let task_id = 1;
            self.tasks.insert(
                task_id,
                Task {
                    id: task_id,
                    type_: type_.to_string(),
                    params: params.to_string(),
                },
            );
            Ok(task_id)
        }
    }

    pub fn start(&self) -> Result<(), MaaError> {
        #[cfg(feature = "with-maa-core")]
        {
            unsafe {
                match AsstStart(self.handle) {
                    1 => Ok(()),
                    _ => Err(MaaError::Unknown),
                }
            }
        }
        
        #[cfg(not(feature = "with-maa-core"))]
        {
            Ok(()) // 模拟成功
        }
    }

    pub fn stop(&self) -> Result<(), MaaError> {
        #[cfg(feature = "with-maa-core")]
        {
            unsafe {
                match AsstStop(self.handle) {
                    1 => Ok(()),
                    _ => Err(MaaError::Unknown),
                }
            }
        }
        
        #[cfg(not(feature = "with-maa-core"))]
        {
            Ok(()) // 模拟成功
        }
    }

    pub fn get_uuid(&self) -> Result<String, MaaError> {
        #[cfg(feature = "with-maa-core")]
        {
            unsafe {
                let mut buff_size = 1024;
                loop {
                    if buff_size > 1024 * 1024 {
                        return Err(MaaError::TooLargeAlloc);
                    }
                    let mut buff: Vec<u8> = Vec::with_capacity(buff_size);
                    let data_size = AsstGetUUID(
                        self.handle, 
                        buff.as_mut_ptr() as *mut i8, 
                        buff_size as u64
                    );
                    if data_size == Self::get_null_size() {
                        buff_size = 2 * buff_size;
                        continue;
                    }
                    buff.set_len(data_size as usize);
                    let ret = String::from_utf8_lossy(&buff).to_string();
                    return Ok(ret);
                }
            }
        }
        
        #[cfg(not(feature = "with-maa-core"))]
        {
            Ok("stub-uuid-12345".to_string())
        }
    }

    pub fn running(&self) -> bool {
        #[cfg(feature = "with-maa-core")]
        {
            unsafe { AsstRunning(self.handle) == 1 }
        }
        
        #[cfg(not(feature = "with-maa-core"))]
        {
            false // 模拟为未运行
        }
    }

    pub fn handle(&self) -> AsstHandle {
        self.handle
    }

    #[allow(dead_code)]
    fn get_null_size() -> u64 {
        #[cfg(feature = "with-maa-core")]
        {
            unsafe { AsstGetNullSize() }
        }
        
        #[cfg(not(feature = "with-maa-core"))]
        {
            0 // 模拟值
        }
    }
}

impl Drop for SafeMaaWrapper {
    fn drop(&mut self) {
        #[cfg(feature = "with-maa-core")]
        {
            unsafe { AsstDestroy(self.handle) }
        }
    }
}

impl SafeMaaWrapper {
    /// 加载 MAA 资源文件（静态方法）
    pub fn load_resource(_path: &str) -> Result<(), MaaError> {
        #[cfg(feature = "with-maa-core")]
        {
            let c_path = CString::new(path)?;
            unsafe {
                match AsstLoadResource(c_path.as_ptr()) {
                    1 => Ok(()),
                    _ => Err(MaaError::Unknown),
                }
            }
        }
        
        #[cfg(not(feature = "with-maa-core"))]
        {
            Ok(()) // 模拟成功
        }
    }
}

// 安全地实现 Send 和 Sync
// SAFETY: MAA 官方库是线程安全的，包装器只是添加了 Rust 类型安全
unsafe impl Send for SafeMaaWrapper {}
unsafe impl Sync for SafeMaaWrapper {}