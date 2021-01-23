use std::fmt;
use std::io;
use std::mem::{self, MaybeUninit};
use std::num::ParseIntError;
use std::ptr::{self, NonNull};
use std::str::FromStr;
use winapi::shared::minwindef::{BOOL, DWORD, FALSE, HMODULE, PDWORD, ULONG};
use winapi::shared::ntdef::PVOID;
use winapi::shared::winerror::NO_ERROR;
use winapi::shared::ws2def::AF_INET;
use winapi::{ENUM, STRUCT};

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

const SCAN_START: usize = 0x0000000000000000;
const SCAN_END: usize = 0x00007fffffffffff;

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

// Steps using Cheat Engine:
// 1. Find life (4 bytes integer, scan for it, get hit, next scan...).
// 2. Once you have two values view their memory. Pick the one with (current life, max life, max life, current es, max es).
// 3. Generate pointermap.
// 4. Relog (or change character, or restart the game).
// 5. Repeat steps 1 and 2.
// 6. Pointer scan for this address. Compare results with other saved pointermap(s). Select address.
// 7. Done! Double-click on your favourite (shorter?) pointer map and note the offsets here.
//
// Do the same for mana.
pub struct PtrMap {
    offsets: Vec<usize>,
}

impl fmt::Debug for PtrMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[base")?;
        for offset in self.offsets.iter() {
            write!(f, " -> {:x}", offset)?;
        }
        f.write_str("]")
    }
}

impl FromStr for PtrMap {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.replace(",", "")
            .split_whitespace()
            .map(|x| {
                if x.starts_with("0x") || x.starts_with("0X") {
                    usize::from_str_radix(&x[2..], 16)
                } else {
                    x.parse::<usize>()
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|offsets| Self { offsets })
    }
}

pub struct Process {
    pub pid: u32,
    handle: NonNull<winapi::ctypes::c_void>,
}

impl Process {
    pub fn open_by_name(starts_with: &str) -> Option<Process> {
        let mut size = 0;
        let mut pids = Vec::<DWORD>::with_capacity(1024);
        if unsafe {
            winapi::um::psapi::EnumProcesses(
                pids.as_mut_ptr(),
                (pids.capacity() * mem::size_of::<DWORD>()) as u32,
                &mut size,
            )
        } == FALSE
        {
            return None;
        }

        for i in 0..(size as usize / mem::size_of::<DWORD>()).min(pids.capacity()) {
            let pid = unsafe { *pids.get_unchecked(i) };
            if pid != 0 {
                match Process::open(pid) {
                    Ok(proc) => match proc.name() {
                        Ok(name) => {
                            if name.starts_with(starts_with) {
                                return Some(proc);
                            }
                        }
                        _ => continue,
                    },
                    _ => continue,
                }
            }
        }
        None
    }

    pub fn open(pid: u32) -> io::Result<Self> {
        NonNull::new(unsafe {
            winapi::um::processthreadsapi::OpenProcess(
                winapi::um::winnt::PROCESS_QUERY_INFORMATION | winapi::um::winnt::PROCESS_VM_READ,
                FALSE,
                pid,
            )
        })
        .map(|handle| Self { pid, handle })
        .ok_or(io::Error::last_os_error())
    }

    pub fn base_addr(&self) -> io::Result<HMODULE> {
        let mut size = 0u32;
        let mut module = MaybeUninit::<HMODULE>::uninit();

        if unsafe {
            winapi::um::psapi::EnumProcessModules(
                self.handle.as_ptr(),
                module.as_mut_ptr(),
                mem::size_of::<HMODULE>() as u32,
                &mut size,
            )
        } == FALSE
        {
            return Err(io::Error::last_os_error());
        }

        Ok(unsafe { module.assume_init() })
    }

    pub fn name(&self) -> io::Result<String> {
        let mut buffer = Vec::with_capacity(64);
        let length = unsafe {
            winapi::um::psapi::GetModuleBaseNameA(
                self.handle.as_ptr(),
                self.base_addr()?,
                buffer.as_mut_ptr(),
                buffer.capacity() as u32,
            )
        };
        if length != 0 {
            unsafe { buffer.set_len(length as usize) };
            String::from_utf8(buffer.iter().map(|b| *b as u8).collect())
                .map_err(|_| io::Error::last_os_error())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub fn memory_regions(&self) -> Vec<winapi::um::winnt::MEMORY_BASIC_INFORMATION> {
        let mut base = SCAN_START;
        let mut regions = Vec::new();
        let mut info = MaybeUninit::uninit();

        while base < SCAN_END {
            let written = unsafe {
                winapi::um::memoryapi::VirtualQueryEx(
                    self.handle.as_ptr(),
                    base as *const _,
                    info.as_mut_ptr(),
                    mem::size_of::<winapi::um::winnt::MEMORY_BASIC_INFORMATION>(),
                )
            };
            if written == 0 {
                break;
            }
            let info = unsafe { info.assume_init() };
            base = info.BaseAddress as usize + info.RegionSize;
            regions.push(info);
        }

        regions
    }

    pub fn read<T>(&self, addr: usize) -> io::Result<T> {
        let mut result = MaybeUninit::uninit();
        let mut read = 0usize;

        if unsafe {
            winapi::um::memoryapi::ReadProcessMemory(
                self.handle.as_ptr(),
                addr as *const _,
                &mut result as *mut _ as _,
                mem::size_of::<T>(),
                &mut read,
            )
        } == FALSE
        {
            Err(io::Error::last_os_error())
        } else {
            Ok(unsafe { result.assume_init() })
        }
    }

    pub fn deref<T>(&self, map: &PtrMap) -> io::Result<T> {
        let base = map
            .offsets
            .iter()
            .take(map.offsets.len() - 1)
            .fold(self.base_addr().map(|b| b as usize), |base, offset| {
                self.read::<usize>(base? + offset)
            })?;

        self.read(base + map.offsets[map.offsets.len() - 1])
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe { winapi::um::handleapi::CloseHandle(self.handle.as_ptr()) };
    }
}

pub fn kill_network(pid: u32) -> Result<usize, u32> {
    unsafe {
        let start = std::time::Instant::now();

        let mut size = 0;
        GetExtendedTcpTable(
            ptr::null_mut(),
            &mut size,
            0,
            AF_INET as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        // TODO consider using std::alloc::alloc
        // See https://en.wikipedia.org/wiki/Flexible_array_member
        // "It is common to allocate `sizeof(struct) + array_len*sizeof(array element)` bytes."
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
