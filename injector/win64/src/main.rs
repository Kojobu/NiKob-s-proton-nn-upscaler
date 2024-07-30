extern crate winapi;

use std::ffi::CString;
use std::ptr;
use winapi::um::libloaderapi::GetProcAddress;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::memoryapi::VirtualAllocEx;
use winapi::um::memoryapi::WriteProcessMemory;
use winapi::um::processthreadsapi::CreateRemoteThread;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::PROCESS_ALL_ACCESS;
use winapi::um::handleapi::CloseHandle;
use winapi::um::winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};

fn inject_dll(process_id: u32, dll_path: &str) {
    let h_process = unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, process_id) };
    if h_process.is_null() {
        println!("Failed to open process");
        return;
    }

    let dll_path_cstring = CString::new(dll_path).unwrap();
    let path_len = dll_path_cstring.as_bytes_with_nul().len();

    let remote_memory = unsafe {
        VirtualAllocEx(h_process, ptr::null_mut(), path_len, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE)
    };

    if remote_memory.is_null() {
        println!("Failed to allocate memory in target process");
        unsafe { CloseHandle(h_process) };
        return;
    }

    unsafe {
        WriteProcessMemory(h_process, remote_memory, dll_path_cstring.as_ptr() as *const _, path_len, ptr::null_mut());

        let h_kernel32 = GetModuleHandleA(b"kernel32.dll\0".as_ptr() as *const i8);
        let load_library_addr = GetProcAddress(h_kernel32, b"LoadLibraryA\0".as_ptr() as *const i8);

        CreateRemoteThread(
            h_process,
            ptr::null_mut(),
            0,
            Some(std::mem::transmute(load_library_addr)),
            remote_memory,
            0,
            ptr::null_mut(),
        );
        CloseHandle(h_process);
    }
}

fn main() {
    let process_id = 1234; // Replace with the actual process ID of the target application
    let dll_path = "C:\\path\\to\\your\\compiled\\dll.dll";
    inject_dll(process_id, dll_path);
}