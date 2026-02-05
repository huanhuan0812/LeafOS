/**
 * leafOS - 内核主函数 (UEFI版本)
 * 
 * 这是操作系统的入口点，由UEFI固件调用
 * 参数由UEFI规范定义：
 *   SystemTable - 指向UEFI系统表的指针
 *   ImageHandle - 当前镜像的句柄
 */

#include "uefi.hpp"
#include <stdint.h>

// ============================================
// 全局变量声明
// ============================================

// 保存UEFI系统表和镜像句柄
extern "C" {
    EFI_SYSTEM_TABLE* gSystemTable = nullptr;
    EFI_HANDLE gImageHandle = nullptr;
    EFI_BOOT_SERVICES* gBootServices = nullptr;
}

// ============================================
// 内核全局构造函数和析构函数处理
// ============================================

// 全局构造函数数组（由链接器提供）
typedef void (*constructor_func)();
extern constructor_func __init_array_start[];
extern constructor_func __init_array_end[];

// 全局析构函数数组
typedef void (*destructor_func)();
extern destructor_func __fini_array_start[];
extern destructor_func __fini_array_end[];

/**
 * @brief 调用所有全局构造函数
 * 
 * 在main函数之前调用，初始化C++全局对象
 * 这个函数由启动汇编代码调用
 */
extern "C" void _init() {
    // 遍历构造函数数组并调用每个构造函数
    for (constructor_func* ctor = __init_array_start; 
         ctor < __init_array_end; 
         ctor++) {
        if (*ctor) {
            (*ctor)();
        }
    }
}

/**
 * @brief 调用所有全局析构函数
 * 
 * 在main函数返回后调用，清理C++全局对象
 * 注意：在操作系统内核中可能永远不会被调用
 */
extern "C" void _fini() {
    // 遍历析构函数数组并调用每个析构函数
    for (destructor_func* dtor = __fini_array_start; 
         dtor < __fini_array_end; 
         dtor++) {
        if (*dtor) {
            (*dtor)();
        }
    }
}

// ============================================
// 简单的调试输出函数
// ============================================

/**
 * @brief 向串口输出一个字符
 * 
 * @param c 要输出的字符
 * 
 * 这是最基本的调试输出，在UEFI控制台可用之前使用
 * 假设使用COM1端口 (0x3F8)
 */
static void debug_putc(char c) {
    // 使用串口COM1进行调试输出
    volatile uint8_t* uart = (volatile uint8_t*)0x3F8;
    
    // 等待发送缓冲区空
    while (!(*uart & 0x20));
    
    // 发送字符
    *uart = c;
    
    // 如果是换行符，还需要发送回车符
    if (c == '\n') {
        debug_putc('\r');
    }
}

/**
 * @brief 向串口输出一个字符串
 * 
 * @param str 要输出的字符串
 */
static void debug_puts(const char* str) {
    while (*str) {
        debug_putc(*str++);
    }
}

/**
 * @brief 向UEFI控制台输出字符串
 * 
 * @param str 要输出的字符串（UTF-16格式）
 * 
 * 使用UEFI提供的控制台输出服务
 * 只有在UEFI环境初始化后才能使用
 */
static void uefi_print(const char16_t* str) {
    if (gSystemTable && gSystemTable->ConOut) {
        // 调用UEFI的字符串输出函数
        gSystemTable->ConOut->OutputString(gSystemTable->ConOut, (CHAR16*)str);
    }
}

// ============================================
// 内存操作辅助函数
// ============================================

/**
 * @brief 内存复制函数
 * 
 * @param dest 目标地址
 * @param src 源地址
 * @param n 字节数
 * @return void* 目标地址
 */
void* memcpy(void* dest, const void* src, size_t n) {
    uint8_t* d = (uint8_t*)dest;
    const uint8_t* s = (const uint8_t*)src;
    
    while (n--) {
        *d++ = *s++;
    }
    
    return dest;
}

/**
 * @brief 内存设置函数
 * 
 * @param s 内存地址
 * @param c 要设置的值
 * @param n 字节数
 * @return void* 内存地址
 */
void* memset(void* s, int c, size_t n) {
    uint8_t* p = (uint8_t*)s;
    
    while (n--) {
        *p++ = (uint8_t)c;
    }
    
    return s;
}

/**
 * @brief 获取字符串长度
 * 
 * @param str 字符串
 * @return size_t 长度
 */
size_t strlen(const char* str) {
    size_t len = 0;
    while (str[len]) {
        len++;
    }
    return len;
}

// ============================================
// 内核模块初始化函数
// ============================================

/**
 * @brief 初始化内存管理模块
 * 
 * @return bool 初始化是否成功
 * 
 * 从UEFI获取内存映射，初始化页表等
 */
bool init_memory_manager() {
    debug_puts("[内存] 正在初始化内存管理器...\n");
    
    // TODO: 从UEFI获取内存映射
    // TODO: 初始化物理内存分配器
    // TODO: 设置虚拟内存分页
    
    debug_puts("[内存] 内存管理器初始化完成\n");
    return true;
}

/**
 * @brief 初始化设备驱动模块
 * 
 * @return bool 初始化是否成功
 */
bool init_device_drivers() {
    debug_puts("[设备] 正在初始化设备驱动...\n");
    
    // TODO: 初始化串口、键盘、鼠标等设备
    
    debug_puts("[设备] 设备驱动初始化完成\n");
    return true;
}

/**
 * @brief 初始化文件系统模块
 * 
 * @return bool 初始化是否成功
 */
bool init_file_system() {
    debug_puts("[文件系统] 正在初始化文件系统...\n");
    
    // TODO: 初始化虚拟文件系统
    // TODO: 挂载根文件系统
    
    debug_puts("[文件系统] 文件系统初始化完成\n");
    return true;
}

/**
 * @brief 初始化图形界面模块
 * 
 * @return bool 初始化是否成功
 */
bool init_graphics() {
    debug_puts("[图形] 正在初始化图形界面...\n");
    
    // TODO: 初始化图形帧缓冲区
    // TODO: 设置显示模式
    
    debug_puts("[图形] 图形界面初始化完成\n");
    return true;
}

/**
 * @brief 初始化热服务模块
 * 
 * @return bool 初始化是否成功
 * 
 * 热服务包括：任务调度、中断处理、系统调用等
 */
bool init_hot_services() {
    debug_puts("[热服务] 正在初始化热服务...\n");
    
    // TODO: 初始化任务调度器
    // TODO: 设置中断描述符表(IDT)
    // TODO: 初始化系统调用
    
    debug_puts("[热服务] 热服务初始化完成\n");
    return true;
}

// ============================================
// 内核主函数
// ============================================

/**
 * @brief 内核主函数
 * 
 * @param SystemTable UEFI系统表指针
 * @param ImageHandle 当前镜像句柄
 * @return EFI_STATUS 返回状态码
 * 
 * 这是操作系统的入口点，由UEFI固件调用
 * 负责初始化所有子系统并启动操作系统
 */
extern "C" EFI_STATUS main(EFI_SYSTEM_TABLE* SystemTable, 
                           EFI_HANDLE ImageHandle) {
    // ============================================
    // 阶段1: 保存UEFI参数并初始化基本环境
    // ============================================
    
    // 保存UEFI参数到全局变量，供其他模块使用
    gSystemTable = SystemTable;
    gImageHandle = ImageHandle;
    gBootServices = SystemTable->BootServices;
    
    // 调试输出启动信息
    debug_puts("\n\n========================================\n");
    debug_puts("    leafOS 内核 - UEFI启动\n");
    debug_puts("========================================\n\n");
    

    // ============================================
    // 阶段2: 初始化所有内核模块
    // ============================================
    
    debug_puts("[内核] 开始初始化内核模块...\n\n");
    
    // 1. 初始化内存管理器（必须先初始化）
    if (!init_memory_manager()) {
        debug_puts("[错误] 内存管理器初始化失败！\n");
        return EFI_OUT_OF_RESOURCES;
    }
    
    // 2. 初始化设备驱动
    if (!init_device_drivers()) {
        debug_puts("[警告] 部分设备驱动初始化失败，继续启动...\n");
    }
    
    // 3. 初始化文件系统
    if (!init_file_system()) {
        debug_puts("[警告] 文件系统初始化失败，继续启动...\n");
    }
    
    // 4. 初始化图形界面
    if (!init_graphics()) {
        debug_puts("[警告] 图形界面初始化失败，继续启动...\n");
    }
    
    // 5. 初始化热服务
    if (!init_hot_services()) {
        debug_puts("[错误] 热服务初始化失败！\n");
        return EFI_DEVICE_ERROR;
    }
    
    // ============================================
    // 阶段3: 退出UEFI启动服务
    // ============================================
    
    debug_puts("\n[内核] 正在退出UEFI启动服务...\n");
    
    // 获取当前内存映射
    UINTN mapSize = 0;
    UINTN mapKey = 0;
    UINTN descriptorSize = 0;
    UINT32 descriptorVersion = 0;
    
    // 第一次调用获取所需缓冲区大小
    EFI_STATUS status = gBootServices->GetMemoryMap(
        &mapSize, nullptr, &mapKey, &descriptorSize, &descriptorVersion);
    
    if (status != EFI_BUFFER_TOO_SMALL) {
        debug_puts("[错误] 无法获取内存映射大小\n");
        return status;
    }
    
    // TODO: 分配内存用于存储内存映射
    // TODO: 实际获取内存映射
    
    // 退出启动服务，接管系统控制权
    status = gBootServices->ExitBootServices(ImageHandle, mapKey);
    if (EFI_ERROR(status)) {
        debug_puts("[错误] 无法退出启动服务\n");
        return status;
    }
    
    debug_puts("[内核] 已成功退出UEFI启动服务\n");
    
    // ============================================
    // 阶段4: 内核主循环
    // ============================================
    
    debug_puts("\n========================================\n");
    debug_puts("    leafOS 内核启动完成！\n");
    debug_puts("    正在进入内核主循环...\n");
    debug_puts("========================================\n\n");
    
    // 进入无限循环，等待中断和调度任务
    while (true) {
        // TODO: 处理中断
        // TODO: 调度任务
        // TODO: 处理系统调用
        
        // 简单的忙等待（临时）
        for (volatile int i = 0; i < 1000000; i++);
        
        // 输出心跳信号（临时）
        debug_putc('.');
    }
    
    // 理论上永远不会到达这里
    return EFI_SUCCESS;
}

// ============================================
// 简单的断言函数
// ============================================

/**
 * @brief 内核断言函数
 * 
 * @param condition 条件表达式
 * @param file 文件名
 * @param line 行号
 * 
 * 如果条件为假，则输出错误信息并挂起系统
 */
void kernel_assert(bool condition, const char* file, int line) {
    if (!condition) {
        debug_puts("\n\n[内核断言失败] ");
        debug_puts(file);
        debug_puts(":");
        
        // 简单地将行号转换为字符串
        char lineStr[16];
        int i = 0;
        int n = line;
        do {
            lineStr[i++] = (n % 10) + '0';
            n /= 10;
        } while (n > 0);
        
        for (int j = i - 1; j >= 0; j--) {
            debug_putc(lineStr[j]);
        }
        
        debug_puts("\n系统已挂起\n");
        
        // 挂起系统
        while (true) {
            asm volatile ("hlt");
        }
    }
}

// 断言宏
#define KERNEL_ASSERT(expr) kernel_assert((expr), __FILE__, __LINE__)