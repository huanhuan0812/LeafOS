# leafOS 内核构建Makefile (UEFI版本)
# 支持UEFI和传统BIOS启动

# 检测系统
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

# 目标架构（默认为x86_64）
ARCH ?= x86_64

# 启动模式：uefi 或 bios（默认）
BOOT_MODE ?= uefi

# 工具链配置
ifeq ($(ARCH),x86_64)
    PREFIX = x86_64-elf-
    TARGET = x86_64-elf
    CFLAGS_ARCH = -m64 -march=x86-64 -mabi=sysv -mcmodel=kernel -mno-red-zone
    LDFLAGS_ARCH = -m elf_x86_64
    KERNEL_LOAD_ADDR = 0x100000
    UEFI_ENTRY_POINT = efi_main
else ifeq ($(ARCH),aarch64)
    PREFIX = aarch64-elf-
    TARGET = aarch64-elf
    CFLAGS_ARCH = -march=armv8-a -mgeneral-regs-only
    LDFLAGS_ARCH = -m aarch64elf
    KERNEL_LOAD_ADDR = 0x40080000
    UEFI_ENTRY_POINT = efi_main
else
    $(error 不支持的架构: $(ARCH)。请使用 x86_64 或 aarch64)
endif

# UEFI特定配置
ifeq ($(BOOT_MODE),uefi)
    CFLAGS += -DUEFI_BOOT -fshort-wchar
    CXXFLAGS += -DUEFI_BOOT -fshort-wchar
    ENTRY_POINT = _start
    LINKER_SCRIPT = kernel/arch/$(ARCH)/linker_uefi.ld
    OUTPUT_ELF = $(BUILD_DIR)/bootx64.efi
    OUTPUT_BIN = $(BUILD_DIR)/kernel.bin
else
    ENTRY_POINT = _start
    LINKER_SCRIPT = kernel/arch/$(ARCH)/linker.ld
    OUTPUT_ELF = $(BUILD_DIR)/kernel.elf
    OUTPUT_BIN = $(BUILD_DIR)/kernel.bin
endif

# 工具定义
CC = $(PREFIX)gcc
CXX = $(PREFIX)g++
LD = $(PREFIX)ld
AS = $(PREFIX)as
OBJCOPY = $(PREFIX)objcopy
OBJDUMP = $(PREFIX)objdump
READELF = $(PREFIX)readelf
NM = $(PREFIX)nm
GRUB_MKRESCUE = grub-mkrescue
QEMU = qemu-system-$(ARCH)

# 检测工具是否存在
TOOL_CHECK = $(shell which $(1) 2>/dev/null)
ifeq ($(call TOOL_CHECK,$(CC)),)
    $(error 工具链 $(CC) 未找到，请检查安装: brew install $(PREFIX)gcc)
endif

# 编译选项
CFLAGS = -Wall -Wextra -Werror -ffreestanding -nostdlib -fno-stack-protector \
         -fno-pic -fno-pie -fno-builtin -fno-common -fno-omit-frame-pointer \
         -O2 -g $(CFLAGS_ARCH)

CXXFLAGS = $(CFLAGS) -std=c++11 -fno-exceptions -fno-rtti

# 链接选项
LDFLAGS = $(LDFLAGS_ARCH) -nostdlib -static -z max-page-size=0x1000 \
          -T $(LINKER_SCRIPT) -Map=$(BUILD_DIR)/kernel.map

# UEFI特定链接选项
ifeq ($(BOOT_MODE),uefi)
    LDFLAGS += -pie --no-dynamic-linker
endif

# 包含目录
INCLUDES = -I./kernel/include \
           -I./kernel/arch/$(ARCH)/include \
           -I./memory/include \
           -I./devices/include \
           -I./graphic/include \
           -I./fs/include \
           -I./hot/include

# 目录定义
BUILD_DIR = build/$(ARCH)/$(BOOT_MODE)
ISO_DIR = iso/$(ARCH)/$(BOOT_MODE)
BOOT_DIR = $(ISO_DIR)/boot
GRUB_DIR = $(BOOT_DIR)/grub

# 源文件查找
KERNEL_SRCS = $(shell find kernel -name "*.c" -o -name "*.cpp" -o -name "*.S")
MEMORY_SRCS = $(shell find memory -name "*.c" -o -name "*.cpp" -o -name "*.S")
DEVICES_SRCS = $(shell find devices -name "*.c" -o -name "*.cpp" -o -name "*.S")
GRAPHIC_SRCS = $(shell find graphic -name "*.c" -o -name "*.cpp" -o -name "*.S")
FS_SRCS = $(shell find fs -name "*.c" -o -name "*.cpp" -o -name "*.S")
HOT_SRCS = $(shell find hot -name "*.c" -o -name "*.cpp" -o -name "*.S")

# UEFI启动文件
ifeq ($(BOOT_MODE),uefi)
    BOOT_SRCS = boot/$(ARCH)/uefi_boot.S
else
    BOOT_SRCS = boot/$(ARCH)/bios_boot.S
endif

# 所有源文件
ALL_SRCS = $(KERNEL_SRCS) $(MEMORY_SRCS) $(DEVICES_SRCS) $(GRAPHIC_SRCS) \
           $(FS_SRCS) $(HOT_SRCS) $(BOOT_SRCS)

# 目标文件
KERNEL_OBJS = $(patsubst %.c,$(BUILD_DIR)/%.o,$(filter %.c,$(KERNEL_SRCS)))
KERNEL_OBJS += $(patsubst %.cpp,$(BUILD_DIR)/%.o,$(filter %.cpp,$(KERNEL_SRCS)))
KERNEL_OBJS += $(patsubst %.S,$(BUILD_DIR)/%.o,$(filter %.S,$(KERNEL_SRCS)))

MEMORY_OBJS = $(patsubst %.c,$(BUILD_DIR)/%.o,$(filter %.c,$(MEMORY_SRCS)))
DEVICES_OBJS = $(patsubst %.c,$(BUILD_DIR)/%.o,$(filter %.c,$(DEVICES_SRCS)))
GRAPHIC_OBJS = $(patsubst %.c,$(BUILD_DIR)/%.o,$(filter %.c,$(GRAPHIC_SRCS)))
FS_OBJS = $(patsubst %.c,$(BUILD_DIR)/%.o,$(filter %.c,$(FS_SRCS)))
HOT_OBJS = $(patsubst %.c,$(BUILD_DIR)/%.o,$(filter %.c,$(HOT_SRCS)))
BOOT_OBJS = $(patsubst %.S,$(BUILD_DIR)/%.o,$(filter %.S,$(BOOT_SRCS)))

ALL_OBJS = $(KERNEL_OBJS) $(MEMORY_OBJS) $(DEVICES_OBJS) $(GRAPHIC_OBJS) \
           $(FS_OBJS) $(HOT_OBJS) $(BOOT_OBJS)

# 最终输出
ISO_IMAGE = leafos-$(ARCH)-$(BOOT_MODE).iso
EFI_DISK_IMG = leafos-$(ARCH)-uefi.img

# 显示构建信息
$(info ========================================)
$(info leafOS 内核构建)
$(info 架构: $(ARCH))
$(info 启动模式: $(BOOT_MODE))
$(info 工具链: $(PREFIX))
$(info ========================================)

# ========================================
# 构建目标
# ========================================

# 默认目标
.PHONY: all
all: $(OUTPUT_ELF) 

# UEFI模式构建EFI文件
ifeq ($(BOOT_MODE),uefi)
$(OUTPUT_ELF): $(ALL_OBJS) $(LINKER_SCRIPT)
	@echo "链接UEFI内核..."
	@mkdir -p $(dir $@)
	$(LD) $(LDFLAGS) $(ALL_OBJS) -o $@
	$(OBJCOPY) -O binary $@ $(OUTPUT_BIN)
	@echo "UEFI内核已生成: $@"
	@echo "二进制文件: $(OUTPUT_BIN)"
	
.PHONY: uefi-disk
uefi-disk: $(OUTPUT_ELF)
	@echo "创建UEFI启动盘..."
	@mkdir -p $(BUILD_DIR)/efi/boot
	@cp $(OUTPUT_ELF) $(BUILD_DIR)/efi/boot/bootx64.efi
	dd if=/dev/zero of=$(EFI_DISK_IMG) bs=1M count=64
	mkfs.fat -F 32 $(EFI_DISK_IMG)
	mcopy -i $(EFI_DISK_IMG) $(BUILD_DIR)/efi ::
	@echo "UEFI启动盘已创建: $(EFI_DISK_IMG)"
	
.PHONY: run-uefi
run-uefi: uefi-disk
	@echo "在QEMU中启动UEFI leafOS..."
ifeq ($(ARCH),x86_64)
	$(QEMU) -bios /usr/share/edk2/x64/OVMF.fd \
	        -drive file=$(EFI_DISK_IMG),format=raw \
	        -m 512M -serial stdio \
	        -no-shutdown -no-reboot
else ifeq ($(ARCH),aarch64)
	$(QEMU) -M virt -cpu cortex-a53 \
	        -bios /usr/share/edk2/aarch64/QEMU_EFI.fd \
	        -drive file=$(EFI_DISK_IMG),format=raw \
	        -m 512M -serial stdio \
	        -no-shutdown -no-reboot
endif
else
# BIOS模式构建
$(OUTPUT_ELF): $(ALL_OBJS) $(LINKER_SCRIPT)
	@echo "链接BIOS内核..."
	@mkdir -p $(dir $@)
	$(LD) $(LDFLAGS) $(ALL_OBJS) -o $@
	$(OBJCOPY) -O binary $@ $(OUTPUT_BIN)
	@echo "BIOS内核已生成: $@"
	
# BIOS ISO镜像
$(ISO_IMAGE): $(OUTPUT_ELF) grub.cfg
	@echo "创建BIOS ISO镜像..."
	@mkdir -p $(GRUB_DIR)
	@cp $(OUTPUT_ELF) $(BOOT_DIR)/kernel.elf
	@cp grub.cfg $(GRUB_DIR)/grub.cfg
	$(GRUB_MKRESCUE) -o $@ $(ISO_DIR) 2>/dev/null || \
		echo "注意: grub-mkrescue可能失败，尝试使用xorriso替代"
	@echo "BIOS ISO镜像已生成: $@"
endif

# grub配置文件（BIOS模式）
grub.cfg:
	@echo "生成GRUB配置文件..."
	@mkdir -p $(GRUB_DIR)
	@echo 'set timeout=0' > $@
	@echo 'set default=0' >> $@
	@echo '' >> $@
	@echo 'menuentry "leafOS" {' >> $@
	@echo '    multiboot2 /boot/kernel.elf' >> $@
	@echo '    boot' >> $@
	@echo '}' >> $@

# 编译规则
$(BUILD_DIR)/%.o: %.c
	@echo "编译C: $<"
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) $(INCLUDES) -c $< -o $@

$(BUILD_DIR)/%.o: %.cpp
	@echo "编译C++: $<"
	@mkdir -p $(dir $@)
	$(CXX) $(CXXFLAGS) $(INCLUDES) -c $< -o $@

$(BUILD_DIR)/%.o: %.S
	@echo "编译汇编: $<"
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) $(INCLUDES) -D__ASSEMBLY__ -c $< -o $@

# 创建链接脚本（如果没有）
$(LINKER_SCRIPT):
	@echo "警告: 链接脚本不存在，创建默认链接脚本..."
	@mkdir -p $(dir $@)
	@echo '/* 默认链接脚本 for $(ARCH) */' > $@
	@echo 'ENTRY($(ENTRY_POINT))' >> $@
	@echo 'SECTIONS' >> $@
	@echo '{' >> $@
	@echo '    . = $(KERNEL_LOAD_ADDR);' >> $@
	@echo '    .text : { *(.text) }' >> $@
	@echo '    .rodata : { *(.rodata) }' >> $@
	@echo '    .data : { *(.data) }' >> $@
	@echo '    .bss : { *(.bss) *(COMMON) }' >> $@
	@echo '}' >> $@
	@echo "默认链接脚本已生成: $@"

# ========================================
# 运行和测试
# ========================================

.PHONY: run
run: 
ifeq ($(BOOT_MODE),uefi)
	$(MAKE) run-uefi
else
	$(MAKE) $(ISO_IMAGE)
	@echo "在QEMU中启动leafOS..."
ifeq ($(ARCH),x86_64)
	$(QEMU) -cdrom $(ISO_IMAGE) -m 512M -serial stdio \
	        -no-shutdown -no-reboot -d cpu_reset
else ifeq ($(ARCH),aarch64)
	$(QEMU) -M virt -cpu cortex-a53 -m 512M \
	        -kernel $(OUTPUT_ELF) -serial stdio \
	        -no-shutdown -no-reboot
endif
endif

# ========================================
# 其他目标
# ========================================

.PHONY: debug
debug: CFLAGS += -DDEBUG -g3 -O0
debug: CXXFLAGS += -DDEBUG -g3 -O0
debug: clean all
	@echo "调试版本已构建"

.PHONY: gdb
gdb: debug
	@echo "启动QEMU等待GDB连接..."
ifeq ($(BOOT_MODE),uefi)
	$(QEMU) -bios /usr/share/edk2/x64/OVMF.fd \
	        -drive file=$(EFI_DISK_IMG),format=raw \
	        -m 512M -s -S &
	@echo "在另一个终端运行: $(PREFIX)gdb -ex 'target remote localhost:1234' $(OUTPUT_ELF)"
else
	$(QEMU) -cdrom $(ISO_IMAGE) -m 512M -s -S &
	@echo "在另一个终端运行: $(PREFIX)gdb -ex 'target remote localhost:1234' $(OUTPUT_ELF)"
endif

.PHONY: analysis
analysis: $(OUTPUT_ELF)
	@echo "=== 内核分析 ==="
	@echo "文件大小: $$(stat -f%z $(OUTPUT_ELF)) 字节"
	@echo ""
	@echo "=== ELF头信息 ==="
	$(READELF) -h $(OUTPUT_ELF) || echo "readelf不可用"
	@echo ""
	@echo "=== 段信息 ==="
	$(READELF) -S $(OUTPUT_ELF) | head -30 || echo "readelf不可用"
	@echo ""
	@echo "=== 符号表（前20个）==="
	$(NM) -n $(OUTPUT_ELF) 2>/dev/null | head -20 || echo "nm不可用"

.PHONY: disasm
disasm: $(OUTPUT_ELF)
	$(OBJDUMP) -d $(OUTPUT_ELF) > $(BUILD_DIR)/kernel.disasm
	@echo "反汇编已保存到: $(BUILD_DIR)/kernel.disasm"

.PHONY: clean
clean:
	rm -rf build iso *.iso *.img grub.cfg
	@echo "已清理构建文件"

.PHONY: clean-$(ARCH)
clean-$(ARCH):
	rm -rf build/$(ARCH) iso/$(ARCH) leafos-$(ARCH)-*.iso leafos-$(ARCH)-*.img
	@echo "已清理 $(ARCH) 构建文件"

.PHONY: all-modes
all-modes:
	@echo "构建所有启动模式..."
	$(MAKE) BOOT_MODE=bios ARCH=$(ARCH) clean all
	$(MAKE) BOOT_MODE=uefi ARCH=$(ARCH) clean all
	@echo "所有启动模式构建完成"

.PHONY: all-arch
all-arch:
	@echo "构建所有架构..."
	$(MAKE) ARCH=x86_64 all-modes
	$(MAKE) ARCH=aarch64 all-modes
	@echo "所有架构构建完成"

.PHONY: deps
deps:
	@echo "安装依赖..."
	brew install x86_64-elf-binutils x86_64-elf-gcc \
	             aarch64-elf-binutils aarch64-elf-gcc \
	             qemu grub xorriso mtools
	@echo "安装UEFI固件..."
	brew install edk2
	@echo "依赖安装完成"

.PHONY: help
help:
	@echo "leafOS 内核构建系统 (支持UEFI和BIOS)"
	@echo ""
	@echo "用法: make [目标] [ARCH=架构] [BOOT_MODE=启动模式]"
	@echo ""
	@echo "目标:"
	@echo "  all          构建内核（默认）"
	@echo "  run          构建并运行在QEMU中"
	@echo "  uefi-disk    创建UEFI启动盘 (仅UEFI模式)"
	@echo "  run-uefi     运行UEFI版本"
	@echo "  debug        构建调试版本"
	@echo "  gdb          启动调试会话"
	@echo "  analysis     分析内核ELF文件"
	@echo "  disasm       生成反汇编"
	@echo "  clean        清理所有构建文件"
	@echo "  all-modes    构建所有启动模式"
	@echo "  all-arch     构建所有架构版本"
	@echo "  deps         安装依赖（macOS）"
	@echo "  help         显示此帮助"
	@echo ""
	@echo "架构选项 (ARCH=):"
	@echo "  x86_64       64位x86架构（默认）"
	@echo "  aarch64      64位ARM架构"
	@echo ""
	@echo "启动模式选项 (BOOT_MODE=):"
	@echo "  uefi         UEFI启动模式（默认）"
	@echo "  bios         传统BIOS启动模式"
	@echo ""
	@echo "示例:"
	@echo "  make                             # 构建x86_64 UEFI版本"
	@echo "  make ARCH=aarch64               # 构建ARM64 UEFI版本"
	@echo "  make BOOT_MODE=bios             # 构建BIOS版本"
	@echo "  make ARCH=x86_64 BOOT_MODE=bios run  # 构建并运行x86_64 BIOS版本"
	@echo "  make all-modes                  # 构建当前架构的所有启动模式"
	@echo "  make all-arch                   # 构建所有架构和启动模式"

# 包含依赖文件
DEPS = $(ALL_OBJS:.o=.d)
-include $(DEPS)