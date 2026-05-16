use crate::boot;

pub fn kernel_main(boot_params: &boot::memory::MemoryParams , acpi_data: &boot::acpi::AcpiCopiedData) -> ! {
    // 这里是内核的入口函数，内核的初始化和主要逻辑将在这里实现
    // 目前我们只是一个简单的占位符，实际的内核功能将在后续开发中逐步添加
    loop{
        core::hint::spin_loop();
    }
}