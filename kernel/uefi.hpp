/**
 * leafOS - UEFI定义头文件
 * 简化的UEFI类型和函数定义
 * 参考: UEFI Specification 2.8+
 */

#pragma once
#ifndef __LEAFOS_UEFI_H__
#define __LEAFOS_UEFI_H__

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// ============================================
// 基本类型定义 (UEFI 2.8 Spec Section 2.3.1)
// ============================================

// 基础类型
typedef uint8_t     BOOLEAN;
typedef int8_t      INT8;
typedef uint8_t     UINT8;
typedef int16_t     INT16;
typedef uint16_t    UINT16;
typedef int32_t     INT32;
typedef uint32_t    UINT32;
typedef int64_t     INT64;
typedef uint64_t    UINT64;
typedef char16_t    CHAR16;     // UTF-16字符
typedef void        VOID;

// UINTN: 平台相关的无符号整数（通常为指针大小）
#if defined(__x86_64__) || defined(_M_X64) || defined(__aarch64__)
typedef uint64_t    UINTN;
typedef int64_t     INTN;
#else
typedef uint32_t    UINTN;
typedef int32_t     INTN;
#endif

// 处理器原生类型
typedef UINTN       EFI_STATUS; // 状态码类型
typedef VOID*       EFI_HANDLE; // 句柄类型
typedef VOID*       EFI_EVENT;  // 事件类型
typedef UINT64      EFI_LBA;    // 逻辑块地址
typedef UINTN       EFI_TPL;    // 任务优先级级别

// 指针类型
typedef CHAR16*     EFI_STRING; // UTF-16字符串

// 状态码定义
#define EFI_SUCCESS               0      // 成功
#define EFI_LOAD_ERROR            (1)    // 加载错误
#define EFI_INVALID_PARAMETER     (2)    // 无效参数
#define EFI_UNSUPPORTED           (3)    // 不支持
#define EFI_BAD_BUFFER_SIZE       (4)    // 缓冲区大小错误
#define EFI_BUFFER_TOO_SMALL      (5)    // 缓冲区太小
#define EFI_NOT_READY             (6)    // 未就绪
#define EFI_DEVICE_ERROR          (7)    // 设备错误
#define EFI_WRITE_PROTECTED       (8)    // 写保护
#define EFI_OUT_OF_RESOURCES      (9)    // 资源不足
#define EFI_NOT_FOUND             (14)   // 未找到
#define EFI_ABORTED               (21)   // 已中止

// 简单的字符串处理宏
#define EFI_ERROR(Status) ((INTN)(Status) < 0)

// ============================================
// 数据结构定义
// ============================================

// 表头结构 (所有UEFI表的开头)
typedef struct {
    UINT64  Signature;          // 表标识符
    UINT32  Revision;           // 规格版本
    UINT32  HeaderSize;         // 表头大小
    UINT32  CRC32;              // CRC校验
    UINT32  Reserved;           // 保留字段
} EFI_TABLE_HEADER;

// GUID (全局唯一标识符)
typedef struct {
    UINT32  Data1;
    UINT16  Data2;
    UINT16  Data3;
    UINT8   Data4[8];
} EFI_GUID;

// 时间和日期
typedef struct {
    UINT16  Year;       // 1900 - 9999
    UINT8   Month;      // 1 - 12
    UINT8   Day;        // 1 - 31
    UINT8   Hour;       // 0 - 23
    UINT8   Minute;     // 0 - 59
    UINT8   Second;     // 0 - 59
    UINT8   Pad1;       // 填充
    UINT32  Nanosecond; // 0 - 999,999,999
    INT16   TimeZone;   // -1440 to 1440 or 2047
    UINT8   Daylight;   // 夏令时标志
    UINT8   Pad2;       // 填充
} EFI_TIME;

// 内存描述符
typedef struct {
    UINT32  Type;           // 内存类型
    UINT64  PhysicalStart;  // 物理起始地址
    UINT64  VirtualStart;   // 虚拟起始地址
    UINT64  NumberOfPages;  // 页数
    UINT64  Attribute;      // 内存属性
} EFI_MEMORY_DESCRIPTOR;

// ============================================
// 公共定义
// ============================================

// 内存类型定义
typedef enum {
    EfiReservedMemoryType,     // 0 保留内存
    EfiLoaderCode,             // 1 引导加载程序代码
    EfiLoaderData,             // 2 引导加载程序数据
    EfiBootServicesCode,       // 3 启动服务代码
    EfiBootServicesData,       // 4 启动服务数据
    EfiRuntimeServicesCode,    // 5 运行时服务代码
    EfiRuntimeServicesData,    // 6 运行时服务数据
    EfiConventionalMemory,     // 7 常规内存（可用于OS）
    EfiUnusableMemory,         // 8 不可用内存
    EfiACPIReclaimMemory,      // 9 ACPI可回收内存
    EfiACPIMemoryNVS,          // 10 ACPI NVS内存
    EfiMemoryMappedIO,         // 11 内存映射IO
    EfiMemoryMappedIOPortSpace,// 12 内存映射IO端口空间
    EfiPalCode,                // 13 PAL代码
    EfiPersistentMemory,       // 14 持久内存
    EfiMaxMemoryType           // 15 最大内存类型
} EFI_MEMORY_TYPE;

// AllocatePages类型定义
typedef enum {
    AllocateAnyPages,      // 在任意可用内存分配
    AllocateMaxAddress,    // 在指定最大地址以下分配
    AllocateAddress,       // 在指定地址分配
    MaxAllocateType
} EFI_ALLOCATE_TYPE;

// 物理地址类型
typedef UINT64 EFI_PHYSICAL_ADDRESS;

// ============================================
// 协议接口定义
// ============================================

// 键盘输入键结构
typedef struct {
    UINT16  ScanCode;          // 扫描码
    CHAR16  UnicodeChar;       // Unicode字符
} EFI_INPUT_KEY;

// 简单文本输入协议（用于键盘输入）
typedef struct EFI_SIMPLE_TEXT_INPUT_PROTOCOL {
    // 重置设备
    EFI_STATUS (*Reset)(struct EFI_SIMPLE_TEXT_INPUT_PROTOCOL* This,
                       BOOLEAN ExtendedVerification);
    
    // 读取按键
    EFI_STATUS (*ReadKeyStroke)(struct EFI_SIMPLE_TEXT_INPUT_PROTOCOL* This,
                               EFI_INPUT_KEY* Key);
    
    // 等待按键事件
    EFI_EVENT  WaitForKey;
} EFI_SIMPLE_TEXT_INPUT_PROTOCOL;

// 控制台模式信息
typedef struct {
    INT32   MaxMode;           // 最大模式数
    INT32   Mode;              // 当前模式
    INT32   Attribute;         // 当前属性
    INT32   CursorColumn;      // 光标列
    INT32   CursorRow;         // 光标行
    BOOLEAN CursorVisible;     // 光标是否可见
} SIMPLE_TEXT_OUTPUT_MODE;

// 简单文本输出协议（用于控制台输出）
typedef struct EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL {
    // 重置设备
    EFI_STATUS (*Reset)(struct EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This,
                       BOOLEAN ExtendedVerification);
    
    // 输出字符串
    EFI_STATUS (*OutputString)(struct EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This,
                              CHAR16* String);
    
    // 测试字符串输出
    EFI_STATUS (*TestString)(struct EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This,
                            CHAR16* String);
    
    // 查询模式信息
    EFI_STATUS (*QueryMode)(struct EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This,
                           UINTN ModeNumber,
                           UINTN* Columns,
                           UINTN* Rows);
    
    // 设置模式
    EFI_STATUS (*SetMode)(struct EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This,
                         UINTN ModeNumber);
    
    // 设置属性
    EFI_STATUS (*SetAttribute)(struct EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This,
                              UINTN Attribute);
    
    // 清除屏幕
    EFI_STATUS (*ClearScreen)(struct EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This);
    
    // 设置光标位置
    EFI_STATUS (*SetCursorPosition)(struct EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This,
                                   UINTN Column,
                                   UINTN Row);
    
    // 启用/禁用光标
    EFI_STATUS (*EnableCursor)(struct EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This,
                              BOOLEAN Visible);
    
    // 当前模式
    SIMPLE_TEXT_OUTPUT_MODE* Mode;
} EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL;

// // 键盘输入键结构
// typedef struct {
//     UINT16  ScanCode;          // 扫描码
//     CHAR16  UnicodeChar;       // Unicode字符
// } EFI_INPUT_KEY;

typedef struct {
    EFI_TABLE_HEADER  Hdr;    // 表头
    
    // 任务优先级服务
    EFI_TPL (*RaiseTPL)(EFI_TPL NewTpl);
    VOID (*RestoreTPL)(EFI_TPL OldTpl);
    
    // 内存分配服务
    EFI_STATUS (*AllocatePages)(EFI_ALLOCATE_TYPE Type,
                               EFI_MEMORY_TYPE MemoryType,
                               UINTN Pages,
                               EFI_PHYSICAL_ADDRESS* Memory);
    EFI_STATUS (*FreePages)(EFI_PHYSICAL_ADDRESS Memory,
                           UINTN Pages);
    EFI_STATUS (*GetMemoryMap)(UINTN* MemoryMapSize,
                              EFI_MEMORY_DESCRIPTOR* MemoryMap,
                              UINTN* MapKey,
                              UINTN* DescriptorSize,
                              UINT32* DescriptorVersion);
    
    // 退出启动服务
    EFI_STATUS (*ExitBootServices)(EFI_HANDLE ImageHandle,
                                  UINTN MapKey);
    
    // 还有很多其他函数...
} EFI_BOOT_SERVICES;


typedef struct {
    EFI_TABLE_HEADER  Hdr;    // 表头
    
    // 获取当前时间
    EFI_STATUS (*GetTime)(EFI_TIME* Time,
                         VOID* Capabilities);
    
    // 设置当前时间
    EFI_STATUS (*SetTime)(EFI_TIME* Time);
    
    // 还有很多其他函数...
}EFI_RUNTIME_SERVICES;

typedef struct {
    EFI_GUID    VendorGuid;     // 供应商GUID
    VOID*       VendorTable;    // 供应商表指针
} EFI_CONFIGURATION_TABLE;

// ============================================
// 系统表结构 (UEFI的核心数据结构)
// ============================================

typedef struct {
    EFI_TABLE_HEADER                  Hdr;        // 表头
    
    // 固件信息
    CHAR16*                           FirmwareVendor;     // 固件厂商
    UINT32                            FirmwareRevision;   // 固件版本
    
    // 控制台句柄
    EFI_HANDLE                        ConsoleInHandle;    // 输入控制台
    EFI_SIMPLE_TEXT_INPUT_PROTOCOL*   ConIn;             // 输入协议
    
    EFI_HANDLE                        ConsoleOutHandle;   // 输出控制台
    EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL*  ConOut;            // 输出协议
    
    EFI_HANDLE                        StandardErrorHandle; // 错误输出
    EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL*  StdErr;            // 错误协议
    
    // 运行时服务
    
    EFI_RUNTIME_SERVICES*             RuntimeServices;   // 运行时服务表
    
    // 启动服务
    EFI_BOOT_SERVICES*                BootServices;      // 启动服务表
    
    // 配置表
    UINTN                             NumberOfTableEntries; // 配置表数量
    EFI_CONFIGURATION_TABLE*          ConfigurationTable;   // 配置表数组
    
} EFI_SYSTEM_TABLE;

// ============================================
// 启动服务表
// ============================================





#ifdef __cplusplus
} // extern "C"
#endif

#endif // __LEAFOS_UEFI_H__