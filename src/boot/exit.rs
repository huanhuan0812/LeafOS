pub fn exit_uefi(){
    // 退出UEFI引导环境，进入操作系统内核
    // 这里可以执行一些清理工作，例如关闭UEFI服务、释放资源等
    // 最后调用UEFI的ExitBootServices函数退出引导环境
    uefi::println!("Exiting UEFI boot services...");
    let _ = unsafe {
        uefi::boot::exit_boot_services(None);
    };
    // 此后的代码将由操作系统内核接管，UEFI引导环境已经退出
    // 除UEFI Runtime Services外，UEFI引导环境的所有服务都将不可用

    // 下一步要进入内核，将会在下一个函数中实现
}