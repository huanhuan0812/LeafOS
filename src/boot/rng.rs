use uefi::prelude::*;
use uefi::proto::rng::{Rng, RngAlgorithmType};

pub fn get_random_seed() -> Result<[u8; 32], uefi::Error> {
    // 获取 RNG 协议的句柄
    let rng_handle = boot::get_handle_for_protocol::<Rng>()
        .expect("RNG protocol not available");
    
    // 打开协议
    let mut rng = boot::open_protocol_exclusive::<Rng>(rng_handle)?;
    
    // 准备缓冲区（32字节种子）
    let mut seed = [0u8; 32];
    
    // 使用默认算法（传 None）或指定算法
    rng.get_rng(None, &mut seed)?;
    
    Ok(seed)
}

// 指定具体算法的版本
pub fn get_random_seed_with_algorithm(
    algorithm: RngAlgorithmType
) -> Result<[u8; 32], uefi::Error> {
    let rng_handle = boot::get_handle_for_protocol::<Rng>()?;
    
    let mut rng = boot::open_protocol_exclusive::<Rng>(rng_handle)?;
    
    let mut seed = [0u8; 32];
    rng.get_rng(Some(algorithm), &mut seed)?;
    
    Ok(seed)
}