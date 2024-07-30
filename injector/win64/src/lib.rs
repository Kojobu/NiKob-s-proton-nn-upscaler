extern crate winapi;
extern crate minhook;
extern crate widestring;
extern crate lazy_static;



// use minhook::{Hook, HookBuilder};

use minhook::MinHook;
use lazy_static::lazy_static;




use std::sync::Mutex;
use winapi::shared::dxgi::IDXGISwapChain;
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, UINT};
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
use winapi::um::winnt::HRESULT;


type PresentFn = unsafe extern "system" fn(*mut IDXGISwapChain, UINT, UINT) -> HRESULT;
lazy_static! {
    static ref ORIGINAL_PRESENT: Mutex<Option<PresentFn>> = Mutex::new(None);
}

unsafe extern "system" fn hooked_present(swap_chain: *mut IDXGISwapChain, sync_interval: UINT, flags: UINT) -> HRESULT {
    println!("Present called!");

    // Call the original Present function
    let original_present = *ORIGINAL_PRESENT.lock().unwrap();
    if let Some(original) = original_present {
        original(swap_chain, sync_interval, flags)
    } else {
        winapi::shared::winerror::E_FAIL
    }
}

#[no_mangle]
pub extern "stdcall" fn DllMain(_hinst_dll: HINSTANCE, fdw_reason: DWORD, _: *mut std::ffi::c_void) -> BOOL {
    match fdw_reason {
        winapi::um::winnt::DLL_PROCESS_ATTACH => {
            println!("DLL_PROCESS_ATTACH: Initializing DirectX hook");

            let lib_dxgi = unsafe { GetModuleHandleA(b"dxgi.dll\0".as_ptr() as *const i8) };
            if lib_dxgi.is_null() {
                println!("Failed to load dxgi.dll");
                return 0;
            }

            let original_present = unsafe { GetProcAddress(lib_dxgi, b"Present\0".as_ptr() as *const i8) };
            if original_present.is_null() {
                println!("Failed to find Present in dxgi.dll");
                return 0;
            }

            let _hook = unsafe {
                MinHook::create_hook(original_present as *mut _, hooked_present as *mut _).expect("Failed to create hook")
            };

            unsafe { MinHook::enable_all_hooks().expect("Failed") };

            // unsafe {
            //     *ORIGINAL_PRESENT.lock().unwrap() = Some(std::mem::transmute::<_, fn() -> PresentFn>(hook));   
            // };
        }
        winapi::um::winnt::DLL_PROCESS_DETACH => {
            println!("DLL_PROCESS_DETACH: Cleaning up DirectX hook");
            // Cleanup logic here if needed
        }
        _ => {}
    }
    1
}
