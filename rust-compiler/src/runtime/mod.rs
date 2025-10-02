// Nature语言运行时模块
// 提供引用计数垃圾回收等运行时功能

// Runtime functions for garbage collection
use std::sync::atomic::{AtomicI32, Ordering};

/// 引用计数对象的内存布局
/// 结构：{ref_count: i32, data: T}
#[repr(C)]
pub struct RCHeader {
    pub ref_count: AtomicI32,
}

impl RCHeader {
    pub fn new() -> Self {
        Self {
            ref_count: AtomicI32::new(1),
        }
    }
}

/// 引用计数垃圾回收运行时函数
pub mod gc {
    use super::*;
    use std::alloc::{alloc, dealloc, Layout};

    /// 增加引用计数
    /// 这个函数会被LLVM后端调用
    #[no_mangle]
    pub extern "C" fn __rc_increment(ptr: *mut u8) {
        println!("RC: __rc_increment called with ptr: {:p}", ptr);
        if ptr.is_null() {
            println!("RC: __rc_increment called with null pointer");
            return;
        }

        unsafe {
            let header = ptr as *mut RCHeader;
            let ref_count = &(*header).ref_count;
            let new_count = ref_count.fetch_add(1, Ordering::Relaxed) + 1;
            println!("RC: Incremented ref count to {}", new_count);
        }
    }

    /// 减少引用计数，如果为0则释放内存
    /// 这个函数会被LLVM后端调用
    #[no_mangle]
    pub extern "C" fn __rc_decrement(ptr: *mut u8) {
        println!("RC: __rc_decrement called with ptr: {:p}", ptr);
        if ptr.is_null() {
            println!("RC: __rc_decrement called with null pointer");
            return;
        }

        unsafe {
            let header = ptr as *mut RCHeader;
            let ref_count = &(*header).ref_count;
            let new_count = ref_count.fetch_sub(1, Ordering::Relaxed) - 1;
            println!("RC: Decremented ref count to {}", new_count);

            if new_count <= 0 {
                println!("RC: Freeing object at {:p}", ptr);
                // 获取对象大小（这里需要知道具体的数据类型大小）
                // 暂时使用一个估算值，实际实现中需要更精确的计算
                let layout = Layout::from_size_align(8, 4).unwrap(); // 假设对象大小为8字节
                dealloc(ptr, layout);
            }
        }
    }

    /// 赋值时管理引用计数（先减后增）
    /// 这个函数会被LLVM后端调用
    #[no_mangle]
    pub extern "C" fn __rc_assign(old_ptr: *mut u8, new_ptr: *mut u8) {
        println!("RC: __rc_assign called with old_ptr: {:p}, new_ptr: {:p}", old_ptr, new_ptr);
        
        // 先减少旧值的引用计数
        if !old_ptr.is_null() {
            __rc_decrement(old_ptr);
        }

        // 再增加新值的引用计数
        if !new_ptr.is_null() {
            __rc_increment(new_ptr);
        }
    }

    /// 创建引用计数对象
    /// 这个函数用于在Rust代码中创建RC对象
    pub fn create_rc_object<T>(data: T) -> *mut u8 {
        // 计算布局：RCHeader + T
        let header_layout = Layout::new::<RCHeader>();
        let data_layout = Layout::new::<T>();
        let (layout, data_offset) = header_layout.extend(data_layout).unwrap();

        unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                panic!("Failed to allocate memory for RC object");
            }

            // 初始化RCHeader
            let header_ptr = ptr as *mut RCHeader;
            std::ptr::write(header_ptr, RCHeader::new());

            // 初始化数据
            let data_ptr = ptr.add(data_offset);
            std::ptr::write(data_ptr as *mut T, data);

            ptr
        }
    }

    /// 获取引用计数对象的数据指针
    pub fn get_data_ptr<T>(rc_ptr: *mut u8) -> *mut T {
        if rc_ptr.is_null() {
            return std::ptr::null_mut();
        }

        unsafe {
            let header_layout = Layout::new::<RCHeader>();
            let data_offset = header_layout.size();
            rc_ptr.add(data_offset) as *mut T
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rc_increment_decrement() {
        // 创建一个简单的RC对象
        let data = 42i32;
        let ptr = gc::create_rc_object(data);
        
        // 测试增加引用计数
        gc::__rc_increment(ptr);
        gc::__rc_increment(ptr);
        
        // 测试减少引用计数
        gc::__rc_decrement(ptr);
        gc::__rc_decrement(ptr);
        gc::__rc_decrement(ptr); // 这应该释放对象
        
        // 注意：在实际测试中，我们需要更仔细地管理内存
    }
}
