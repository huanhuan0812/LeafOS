#include <efi/efi.h>
#include <efi/efilib.h>

#define UCS2(str) reinterpret_cast<CHAR16*>(const_cast<char16_t*>(u##str))

extern "C"
EFI_STATUS EFIAPI efi_main(EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE *SystemTable){
    InitializeLib(ImageHandle, SystemTable);
    
    CHAR16* str = UCS2("hello");

    Print(str);
    Print(UCS2("\n"));
    
    return EFI_SUCCESS;
}