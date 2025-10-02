// 真正的GC运行时实现
// 这个文件将被编译并与LLVM IR链接

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

// 引用计数对象的内存布局
// 结构：{ref_count: int32_t, data: T}
typedef struct {
    int32_t ref_count;
    // 实际数据紧随其后
} RC_Header;

// 真正的引用计数实现
void __rc_increment_wrapper(void* ptr) {
    if (ptr == NULL) {
        return;
    }
    
    RC_Header* header = (RC_Header*)ptr;
    header->ref_count++;
    
    printf("RC: Incremented ref count to %d for ptr: %p\n", header->ref_count, ptr);
    fflush(stdout);
}

void __rc_decrement_wrapper(void* ptr) {
    if (ptr == NULL) {
        return;
    }
    
    RC_Header* header = (RC_Header*)ptr;
    header->ref_count--;
    
    printf("RC: Decremented ref count to %d for ptr: %p\n", header->ref_count, ptr);
    fflush(stdout);
    
    // 如果引用计数为0，释放内存
    if (header->ref_count <= 0) {
        printf("RC: Freeing object at %p\n", ptr);
        fflush(stdout);
        free(ptr);
    }
}

void __rc_assign_wrapper(void* old_ptr, void* new_ptr) {
    printf("RC: Assign called - old: %p, new: %p\n", old_ptr, new_ptr);
    fflush(stdout);
    
    // 先减少旧值的引用计数
    if (old_ptr != NULL) {
        __rc_decrement_wrapper(old_ptr);
    }
    
    // 再增加新值的引用计数
    if (new_ptr != NULL) {
        __rc_increment_wrapper(new_ptr);
    }
}
