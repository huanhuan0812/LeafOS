cmake_minimum_required(VERSION 3.20)
project(leafOS VERSION 1.0.0 LANGUAGES CXX C)

# EFI架构设置
set(EFI_ARCH "x86_64" CACHE STRING "EFI architecture")

# 查找gnu-efi
find_path(GNUEFI_INCLUDE_DIR
    NAMES efi.h
    PATHS /usr/include/efi /usr/local/include/efi
    REQUIRED
)

find_path(GNUEFI_ARCH_INCLUDE_DIR
    NAMES efibind.h
    PATHS /usr/include/efi/x86_64 /usr/local/include/efi/x86_64
    REQUIRED
)

find_library(GNUEFI_EFI_LIBRARY
    NAMES efi
    PATHS /usr/lib /usr/lib64
    REQUIRED
)

find_library(GNUEFI_GNUEFI_LIBRARY
    NAMES gnuefi
    PATHS /usr/lib /usr/lib64
    REQUIRED
)

find_file(GNUEFI_CRT0
    NAMES crt0-efi-x86_64.o
    PATHS /usr/lib /usr/lib64
    REQUIRED
)

find_file(GNUEFI_LDSCRIPT
    NAMES elf_x86_64_efi.lds
    PATHS /usr/lib /usr/lib64
    REQUIRED
)

# 源代码
set(SOURCES
    kernel/main.cpp
)

# 创建目标
add_executable(${PROJECT_NAME} ${SOURCES})

# 设置编译选项
target_compile_options(${PROJECT_NAME} PRIVATE
    -ffreestanding
    -fshort-wchar
    -fno-stack-protector
    -fno-strict-aliasing
    -fno-merge-constants
    -fno-stack-check
    -fno-omit-frame-pointer
    -fno-common
    -maccumulate-outgoing-args
    -mno-red-zone
    -mno-mmx
    -mno-sse
    -mno-sse2
    -mno-80387
    -mno-fp-ret-in-387
    -mno-stack-arg-probe
    -Wall
    -Wextra
    -I/usr/include/efi
    -I/usr/include/efi/x86_64
    -I/usr/include/efi/protocol
    -DEFI_FUNCTION_WRAPPER
    -DGNU_EFI_USE_MS_ABI
    -Dx86_64
    -m64
)

# C++特定选项
target_compile_options(${PROJECT_NAME} PRIVATE
    $<$<COMPILE_LANGUAGE:CXX>:-fno-exceptions>
    $<$<COMPILE_LANGUAGE:CXX>:-fno-rtti>
    $<$<COMPILE_LANGUAGE:CXX>:-fno-threadsafe-statics>
    $<$<COMPILE_LANGUAGE:CXX>:-fno-use-cxa-atexit>
)

# 关键修复1: 使用正确的链接顺序和选项
target_link_options(${PROJECT_NAME} PRIVATE
    -nostdlib
    -nodefaultlibs
    -Wl,--no-undefined
    -Wl,--build-id=none
    -Wl,--dll
    -Wl,-Bsymbolic
    -Wl,--no-warn-mismatch
    -Wl,--no-pie  # 关键：禁用PIE
    -Wl,-T${GNUEFI_LDSCRIPT}  # 去掉空格
)

# 关键修复2: 正确的库链接顺序
target_link_libraries(${PROJECT_NAME}
    ${GNUEFI_CRT0}
    -L/usr/lib
    -Wl,--start-group       # 2. 使用组解决循环依赖
    ${GNUEFI_GNUEFI_LIBRARY} # 3. gnuefi必须先链接
    ${GNUEFI_EFI_LIBRARY}    # 4. efi后链接
    -Wl,--end-group
)

# 设置输出属性
set_target_properties(${PROJECT_NAME} PROPERTIES
    PREFIX ""
    SUFFIX ".so"
)

# 关键修复3: 生成EFI文件的正确步骤
add_custom_command(TARGET ${PROJECT_NAME} POST_BUILD
    COMMAND ${CMAKE_OBJCOPY} 
        --target=efi-app-x86_64 
        --subsystem=10 
        $<TARGET_FILE:${PROJECT_NAME}> 
        $<TARGET_FILE_DIR:${PROJECT_NAME}>/${PROJECT_NAME}.efi
    COMMAND ${CMAKE_COMMAND} -E echo "Generated: $<TARGET_FILE_DIR:${PROJECT_NAME}>/${PROJECT_NAME}.efi"
    COMMENT "Generating EFI executable"
)

# 打印配置信息
message(STATUS "GNUEFI_CRT0: ${GNUEFI_CRT0}")
message(STATUS "GNUEFI_LDSCRIPT: ${GNUEFI_LDSCRIPT}")
message(STATUS "GNUEFI_GNUEFI_LIBRARY: ${GNUEFI_GNUEFI_LIBRARY}")
message(STATUS "GNUEFI_EFI_LIBRARY: ${GNUEFI_EFI_LIBRARY}")
