use std::mem::{size_of, MaybeUninit};

use winapi::{ENUM, STRUCT};

use winapi::shared::minwindef::{BOOL, DWORD, HMODULE, PDWORD, ULONG};

use winapi::shared::ntdef::PVOID;

use winapi::shared::ws2def::AF_INET;

use winapi::shared::winerror::NO_ERROR;

use winapi::um::psapi::{EnumProcessModules, EnumProcesses, GetModuleBaseNameA};

use winapi::um::processthreadsapi::OpenProcess;

use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

use winapi::um::handleapi::CloseHandle;

const ANY_SIZE: usize = 1;

// From https://github.com/retep998/winapi-rs/pull/802
ENUM! {enum TcpTableClass {
    TCP_TABLE_BASIC_LISTENER = 0,
    TCP_TABLE_BASIC_CONNECTIONS = 1,
    TCP_TABLE_BASIC_ALL = 2,
    TCP_TABLE_OWNER_PID_LISTENER = 3,
    TCP_TABLE_OWNER_PID_CONNECTIONS = 4,
    TCP_TABLE_OWNER_PID_ALL = 5,
    TCP_TABLE_OWNER_MODULE_LISTENER = 6,
    TCP_TABLE_OWNER_MODULE_CONNECTIONS = 7,
    TCP_TABLE_OWNER_MODULE_ALL = 8,
}}
#[allow(non_camel_case_types)]
pub type PTCP_TABLE_CLASS = *mut TcpTableClass;

STRUCT! {#[allow(non_snake_case)] struct MIB_TCPROW_OWNER_PID {
    dwState: DWORD,
    dwLocalAddr: DWORD,
    dwLocalPort: DWORD,
    dwRemoteAddr: DWORD,
    dwRemotePort: DWORD,
    dwOwningPid: DWORD,
}}
#[allow(non_camel_case_types)]
pub type PMIB_TCPROW_OWNER_PID = *mut MIB_TCPROW_OWNER_PID;

STRUCT! {#[allow(non_snake_case)] struct MIB_TCPTABLE_OWNER_PID {
    dwNumEntries: DWORD,
    table: [MIB_TCPROW_OWNER_PID; ANY_SIZE],
}}
#[allow(non_camel_case_types)]
pub type PMIB_TCPTABLE_OWNER_PID = *mut MIB_TCPTABLE_OWNER_PID;

STRUCT! {#[allow(non_snake_case)] struct MIB_TCPROW {
    dwState: DWORD,
    dwLocalAddr: DWORD,
    dwLocalPort: DWORD,
    dwRemoteAddr: DWORD,
    dwRemotePort: DWORD,
}}
#[allow(non_camel_case_types)]
pub type PMIB_TCPROW = *mut MIB_TCPROW;

#[link(name = "iphlpapi")]
extern "system" {
    // https://docs.microsoft.com/en-us/windows/win32/api/iphlpapi/nf-iphlpapi-getextendedtcptable
    pub fn GetExtendedTcpTable(
        pTcpTable: PVOID,
        pdwSize: PDWORD,
        bOrder: BOOL,
        ulAf: ULONG,
        TableClass: TcpTableClass,
        Reserved: ULONG,
    ) -> DWORD;

    pub fn SetTcpEntry(pTcpRow: PMIB_TCPROW) -> DWORD;
}

pub fn kill_network(pid: u32) -> Result<usize, u32> {
    unsafe {
        let start = std::time::Instant::now();

        let mut size = 0;
        GetExtendedTcpTable(
            std::ptr::null_mut(),
            &mut size,
            0,
            AF_INET as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        // TODO consider using std::alloc::alloc
        let mut buffer = Vec::<u8>::with_capacity(size as usize);

        let res = GetExtendedTcpTable(
            buffer.as_mut_ptr() as PVOID,
            &mut size,
            0,
            AF_INET as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );
        if res != NO_ERROR {
            return Err(res);
        }

        let table = buffer.as_mut_ptr() as PMIB_TCPTABLE_OWNER_PID;

        let mut ok = 0;
        for i in 0..(*table).dwNumEntries as usize {
            let entry = (*table).table.get_unchecked(i);
            if entry.dwOwningPid == pid {
                if SetTcpEntry(&mut MIB_TCPROW {
                    dwState: 12, // magic number to terminate
                    dwLocalAddr: entry.dwLocalAddr,
                    dwLocalPort: entry.dwLocalPort,
                    dwRemoteAddr: entry.dwRemoteAddr,
                    dwRemotePort: entry.dwRemotePort,
                }) == NO_ERROR
                {
                    ok += 1;
                }
            }
        }
        if ok == 0 {
            eprintln!("logout err: didn't close any connection!");
        } else {
            eprintln!("logout success: took {:?} for pid {}", start.elapsed(), pid);
        }
        Ok(ok)
    }
}

pub fn get_proc_name(pid: u32) -> Option<String> {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        if !process.is_null() {
            let mut module = MaybeUninit::uninit();
            let mut buffer = Vec::with_capacity(64);
            let mut needed = 0;
            if EnumProcessModules(
                process,
                module.as_mut_ptr(),
                size_of::<HMODULE>() as u32,
                &mut needed,
            ) != 0
            {
                let length = GetModuleBaseNameA(
                    process,
                    module.assume_init(),
                    buffer.as_mut_ptr(),
                    buffer.capacity() as u32,
                );
                if length != 0 {
                    buffer.set_len(length as usize);
                    if let Ok(value) = String::from_utf8(buffer.iter().map(|b| *b as u8).collect())
                    {
                        CloseHandle(process);
                        return Some(value);
                    }
                }
            }
        }
        CloseHandle(process);
        None
    }
}

pub fn find_proc(starts_with: &str) -> Option<u32> {
    unsafe {
        let mut needed = 0;
        let mut pids = Vec::<DWORD>::with_capacity(1024);
        if EnumProcesses(
            pids.as_mut_ptr(),
            (pids.capacity() * size_of::<DWORD>()) as u32,
            &mut needed,
        ) == 0
        {
            return None;
        }

        for i in 0..(needed as usize / size_of::<DWORD>()).min(pids.capacity()) {
            let pid = *pids.get_unchecked(i);
            if pid != 0 {
                if let Some(name) = get_proc_name(pid) {
                    if name.starts_with(starts_with) {
                        return Some(pid);
                    }
                }
            }
        }
        None
    }
}
