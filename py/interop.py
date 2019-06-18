import ctypes
import sys
import time

NO_ERROR = 0

INPUT_MOUSE = 0
INPUT_KEYBOARD = 1
INPUT_HARDWARE = 2

KEYEVENTF_KEYUP = 0x0002

# um/winnt.h
PROCESS_TERMINATE                  = 0x0001
PROCESS_CREATE_THREAD              = 0x0002
PROCESS_SET_SESSIONID              = 0x0004
PROCESS_VM_OPERATION               = 0x0008
PROCESS_VM_READ                    = 0x0010
PROCESS_VM_WRITE                   = 0x0020
PROCESS_DUP_HANDLE                 = 0x0040
PROCESS_CREATE_PROCESS             = 0x0080
PROCESS_SET_QUOTA                  = 0x0100
PROCESS_SET_INFORMATION            = 0x0200
PROCESS_QUERY_INFORMATION          = 0x0400
PROCESS_SUSPEND_RESUME             = 0x0800
PROCESS_QUERY_LIMITED_INFORMATION  = 0x1000
PROCESS_SET_LIMITED_INFORMATION    = 0x2000

KEY_NAMES = [
    'LMB',
    'RMB',
    'BRK',
    'MMB',
    'X1MB',
    'X2MB',
    '(undef)',
    'BACK',
    'TAB',
    '(reserved)',
    '(reserved)',
    'CLEAR',
    'ENTER',
    '(undef)',
    '(undef)',
    'SHIFT',
    'CTRL',
    'ALT',
    'PAUSE',
    'CAPS LOCK',
    'IME Kana',
    'IME Hanguel',
    'IME Hangul',
    '(undef)',
    'IME Junja',
    'IME final',
    'IME Hanja',
    'IME Kanji',
    '(undef)',
    'ESC',
    'IME convert',
    'IME nonconvert',
    'IME accept',
    'IME mode change request',
    'SPACE',
    'PG UP',
    'PG DW',
    'END',
    'HOME',
    'LEFT',
    'UP',
    'RIGHT',
    'DOWN',
    'SELECT',
    'PRINT',
    'EXEC',
    'PSCR',
    'INS',
    'DEL',
    'HELP',
    '0',
    '1',
    '2',
    '3',
    '4',
    '5',
    '6',
    '7',
    '8',
    '9',
    '(undef)',
    '(undef)',
    '(undef)',
    '(undef)',
    '(undef)',
    '(undef)',
    '(undef)',
    'A',
    'B',
    'C',
    'D',
    'E',
    'F',
    'G',
    'H',
    'I',
    'J',
    'K',
    'L',
    'M',
    'N',
    'O',
    'P',
    'Q',
    'R',
    'S',
    'T',
    'U',
    'V',
    'W',
    'X',
    'Y',
    'Z',
    'LWIN',
    'RWIN',
    'APP',
    '(reserved)',
    'SLEEP',
    'NUM0',
    'NUM1',
    'NUM2',
    'NUM3',
    'NUM4',
    'NUM5',
    'NUM6',
    'NUM7',
    'NUM8',
    'NUM9',
    '*',
    '+',
    'SEP',
    '-',
    'DEC',
    '/',
    'F1',
    'F2',
    'F3',
    'F4',
    'F5',
    'F6',
    'F7',
    'F8',
    'F9',
    'F10',
    'F11',
    'F12',
    'F13',
    'F14',
    'F15',
    'F16',
    'F17',
    'F18',
    'F19',
    'F20',
    'F21',
    'F22',
    'F23',
    'F24',
    '(unassign)',
    '(unassign)',
    '(unassign)',
    '(unassign)',
    '(unassign)',
    '(unassign)',
    '(unassign)',
    '(unassign)',
    'NUM LOCK',
    'SCROLL LOCK',
    'OEM specific',
    'OEM specific',
    'OEM specific',
    'OEM specific',
    'OEM specific',
    '(unassign)',
    '(unassign)',
    '(unassign)',
    '(unassign)',
    '(unassign)',
    '(unassign)',
    '(unassign)',
    '(unassign)',
    '(unassign)',
    'LSHIFT',
    'RSHIFT',
    'LCTRL',
    'RCTRL',
    'LMENU',
    'RMENU',
    'Browser Back',
    'Browser Forward',
    'Browser Refresh',
    'Browser Stop',
    'Browser Search',
    'Browser Favorites',
    'Browser Start and Home',
    'VOL MUTE',
    'VOL DOWN',
    'VOL UP',
    'TRACK NXT',
    'TRACK PRV',
    'TRACK STOP',
    'TRACK TOGG',
]


class POINT(ctypes.Structure):
    _fields_ = [('x', ctypes.c_long), ('y', ctypes.c_long)]

    def __str__(self):
        return f'{self.x} {self.y}'


class MOUSEINPUT(ctypes.Structure):
    """
    https://docs.microsoft.com/en-us/windows/desktop/api/winuser/ns-winuser-tagmouseinput
    typedef struct tagMOUSEINPUT {
        LONG      dx;
        LONG      dy;
        DWORD     mouseData;
        DWORD     dwFlags;
        DWORD     time;
        ULONG_PTR dwExtraInfo;
    } MOUSEINPUT, *PMOUSEINPUT, *LPMOUSEINPUT;
    """
    _fields_ = [
        ('dx', ctypes.c_long),
        ('dy', ctypes.c_long),
        ('mouseData', ctypes.c_long),
        ('dwFlags', ctypes.c_long),
        ('time', ctypes.c_long),
        ('dwExtraInfo', ctypes.POINTER(ctypes.c_ulong))
    ]


class KEYBDINPUT(ctypes.Structure):
    """
    https://docs.microsoft.com/en-us/windows/desktop/api/winuser/ns-winuser-tagkeybdinput
    typedef struct tagKEYBDINPUT {
        WORD      wVk;
        WORD      wScan;
        DWORD     dwFlags;
        DWORD     time;
        ULONG_PTR dwExtraInfo;
    } KEYBDINPUT, *PKEYBDINPUT, *LPKEYBDINPUT;
    """
    _fields_ = [
        ('wVk', ctypes.c_short),
        ('wScan', ctypes.c_short),
        ('dwFlags', ctypes.c_long),
        ('time', ctypes.c_long),
        ('dwExtraInfo', ctypes.POINTER(ctypes.c_ulong))
    ]


class HARDWAREINPUT(ctypes.Structure):
    """
    https://docs.microsoft.com/en-us/windows/desktop/api/winuser/ns-winuser-taghardwareinput
    typedef struct tagHARDWAREINPUT {
        DWORD uMsg;
        WORD  wParamL;
        WORD  wParamH;
    } HARDWAREINPUT, *PHARDWAREINPUT, *LPHARDWAREINPUT;
    """
    _fields_ = [
        ('uMsg', ctypes.c_long),
        ('wParamL', ctypes.c_short),
        ('wParamH', ctypes.c_short)
    ]


class INPUTUNION(ctypes.Union):
    # See INPUT
    _fields_ = [('mi', MOUSEINPUT), ('ki', KEYBDINPUT), ('hi', HARDWAREINPUT)]


class INPUT(ctypes.Structure):
    """
    https://docs.microsoft.com/en-us/windows/desktop/api/winuser/ns-winuser-taginput
    typedef struct tagINPUT {
        DWORD type;
        union {
            MOUSEINPUT    mi;
            KEYBDINPUT    ki;
            HARDWAREINPUT hi;
        } DUMMYUNIONNAME;
    } INPUT, *PINPUT, *LPINPUT;
    """
    _fields_ = [('type', ctypes.c_long), ('value', INPUTUNION)]


# noinspection PyPep8Naming
class MIB_TCPROW(ctypes.Structure):
    _fields_ = [
        ('dwState', ctypes.c_long),
        ('dwLocalAddr', ctypes.c_ulong),  # actually long
        ('dwLocalPort', ctypes.c_long),
        ('dwRemoteAddr', ctypes.c_ulong),  # actually long
        ('dwRemotePort', ctypes.c_long)
    ]


# noinspection PyPep8Naming
class MIB_TCPROW_OWNER_PID(ctypes.Structure):
    _fields_ = [
        ('dwState', ctypes.c_long),
        ('dwLocalAddr', ctypes.c_ulong),  # actually long
        ('dwLocalPort', ctypes.c_long),
        ('dwRemoteAddr', ctypes.c_ulong),  # actually long
        ('dwRemotePort', ctypes.c_long),
        ('dwOwningPid', ctypes.c_long)
    ]


# noinspection PyPep8Naming, PyTypeChecker
class MIB_TCPTABLE_OWNER_PID(ctypes.Structure):
    _fields_ = [
        ('dwNumEntries', ctypes.c_long),
        ('table', MIB_TCPROW_OWNER_PID * 0)
    ]

    @property
    def entries(self):
        # cast the zero-sized array into one of the right size
        return ctypes.cast(
            ctypes.byref(self.table),
            ctypes.POINTER((MIB_TCPROW_OWNER_PID * self.dwNumEntries))).contents


def is_admin():
    """
    https://docs.microsoft.com/en-us/windows/desktop/api/shlobj_core/nf-shlobj_core-isuseranadmin
    BOOL IsUserAnAdmin();
    """
    return bool(ctypes.windll.shell32.IsUserAnAdmin())


def elevate(path):
    """
    https://docs.microsoft.com/en-us/windows/desktop/api/shellapi/nf-shellapi-shellexecutew
    HINSTANCE ShellExecuteW(
        HWND    hwnd,
        LPCWSTR lpOperation,
        LPCWSTR lpFile,
        LPCWSTR lpParameters,
        LPCWSTR lpDirectory,
        INT     nShowCmd
    );
    """
    ctypes.windll.shell32.ShellExecuteW(None, "runas", sys.executable, path, None, 1)


def wait_mouse(which):
    """
    https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-getasynckeystate
    SHORT GetAsyncKeyState(
        int vKey
    );
    """
    # 1 for left, 2 for right
    while True:  # wait down
        time.sleep(0.01)
        if ctypes.windll.user32.GetAsyncKeyState(which):
            break

    while True:  # wait release
        time.sleep(0.01)
        if not ctypes.windll.user32.GetAsyncKeyState(which):
            break

    return get_mouse()


def wait_key(before=ctypes.create_string_buffer(256), after=ctypes.create_string_buffer(256)):
    """
    https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-getkeyboardstate
    BOOL GetKeyboardState(
        PBYTE lpKeyState
    );
    """
    # get the current keyboard state
    ctypes.windll.user32.GetKeyState(0)  # needed for some reason
    ctypes.windll.user32.GetKeyboardState(before)
    for i in range(256):
        before[i] = int((before[i][0] & 0x80) != 0)

    while True:  # get the new keyboard state...
        time.sleep(0.05)
        ctypes.windll.user32.GetKeyState(0)  # needed for some reason
        ctypes.windll.user32.GetKeyboardState(after)
        for key, (b, a) in enumerate(zip(before, after)):
            b = b[0]
            a = int((a[0] & 0x80) != 0)
            if b != a:  # ...until we find a difference,
                # then wait until the pressed key is released
                while True:
                    time.sleep(0.05)
                    if not ctypes.windll.user32.GetAsyncKeyState(key):
                        return key


def get_mouse():
    """
    https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-getcursorpos
    BOOL GetCursorPos(
        LPPOINT lpPoint
    );

    Returns x, y as int
    """
    pt = POINT()
    ctypes.windll.user32.GetCursorPos(ctypes.byref(pt))
    return pt.x, pt.y


def get_color(x, y):
    """
    https://docs.microsoft.com/en-us/windows/desktop/api/wingdi/nf-wingdi-getpixel
    COLORREF GetPixel(
        HDC hdc,
        int x,
        int y
    );

    Returns colors as zbgr int
    """
    return ctypes.windll.gdi32.GetPixel(ctypes.windll.user32.GetDC(0), x, y)


def is_down(vk):
    """
    https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-getkeystate
    SHORT GetKeyState(
        int nVirtKey
    );
    """
    return (ctypes.windll.user32.GetKeyState(vk) & 0x80) != 0


def press(vk, down=None):
    """
    https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-sendinput
    UINT SendInput(
        UINT    cInputs,
        LPINPUT pInputs,
        int     cbSize
    );
    """
    if down is None:
        press(vk, True)
        press(vk, False)
        return

    count = ctypes.c_uint(1)
    inputs = INPUT(type=INPUT_KEYBOARD, value=INPUTUNION(ki=KEYBDINPUT(
        wVk=vk,
        wScan=0,
        dwFlags=0 if down else KEYEVENTF_KEYUP,
        time=0,
        dwExtraInfo=None
    )))
    ctypes.windll.user32.SendInput(count, ctypes.byref(inputs), ctypes.sizeof(inputs))


def get_extended_tcp_table():
    """
    https://docs.microsoft.com/en-us/windows/desktop/api/iphlpapi/nf-iphlpapi-getextendedtcptable
    IPHLPAPI_DLL_LINKAGE DWORD GetExtendedTcpTable(
        PVOID           pTcpTable,
        PDWORD          pdwSize,
        BOOL            bOrder,
        ULONG           ulAf,
        TCP_TABLE_CLASS TableClass,
        ULONG           Reserved
    );
    """
    tcp_table = ctypes.c_void_p(None)
    size = ctypes.c_long()
    order = ctypes.c_bool(False)
    af = ctypes.c_long(2)  # AF_INET
    table_class = ctypes.c_int(5)  # TCP_TABLE_OWNER_PID_ALL
    reserved = ctypes.c_long(0)

    # determine size
    ctypes.windll.iphlpapi.GetExtendedTcpTable(tcp_table, ctypes.byref(size), order, af, table_class, reserved)

    # alloc size and get table
    tcp_table = ctypes.create_string_buffer(size.value)
    ret = ctypes.windll.iphlpapi.GetExtendedTcpTable(tcp_table, ctypes.byref(size), order, af, table_class, reserved)
    if ret != NO_ERROR:
        print('failed to get tcp table with exit code', ret, file=sys.stderr)

    # cast allocation as the right type
    return ctypes.cast(tcp_table, ctypes.POINTER(MIB_TCPTABLE_OWNER_PID)).contents


def kill_connection(pid):
    """
    https://docs.microsoft.com/en-us/windows/desktop/api/iphlpapi/nf-iphlpapi-settcpentry
    IPHLPAPI_DLL_LINKAGE DWORD SetTcpEntry(
        PMIB_TCPROW pTcpRow
    );
    """
    ok = True
    for entry in get_extended_tcp_table().entries:
        if entry.dwOwningPid == pid:
            ret = ctypes.windll.iphlpapi.SetTcpEntry(ctypes.byref(MIB_TCPROW(
                dwState=12,  # magic number to terminate the connection
                dwLocalAddr=entry.dwLocalAddr,
                dwLocalPort=entry.dwLocalPort,
                dwRemoteAddr=entry.dwRemoteAddr,
                dwRemotePort=entry.dwRemotePort
            )))
            if ret != NO_ERROR:
                ok = False

    return ok


def get_process_name(pid, base_name=ctypes.create_string_buffer(128)):
    """
    https://docs.microsoft.com/en-us/windows/desktop/api/processthreadsapi/nf-processthreadsapi-openprocess
    HANDLE OpenProcess(
        DWORD dwDesiredAccess,
        BOOL  bInheritHandle,
        DWORD dwProcessId
    );

    https://docs.microsoft.com/en-us/windows/desktop/api/psapi/nf-psapi-enumprocessmodules
    BOOL EnumProcessModules(
        HANDLE  hProcess,
        HMODULE *lphModule,
        DWORD   cb,
        LPDWORD lpcbNeeded
    );

    https://docs.microsoft.com/en-us/windows/desktop/api/Psapi/nf-psapi-getmodulebasenamea
    DWORD GetModuleBaseNameA(
        HANDLE  hProcess,
        HMODULE hModule,
        LPSTR   lpBaseName,
        DWORD   nSize
    );

    https://docs.microsoft.com/en-us/windows/desktop/api/handleapi/nf-handleapi-closehandle
    BOOL CloseHandle(
        HANDLE hObject
    );
    """
    # get a handle to the process
    desired_access = ctypes.c_long(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ)
    inherit_handle = ctypes.c_bool(False)
    process_id = ctypes.c_long(pid)
    process = ctypes.windll.kernel32.OpenProcess(desired_access, inherit_handle, process_id)
    if not process:
        return

    try:
        module = ctypes.c_void_p()
        cb_needed = ctypes.c_long()
        ret = ctypes.windll.psapi.EnumProcessModules(
            process, ctypes.byref(module), ctypes.sizeof(module), ctypes.byref(cb_needed))

        if ret:
            ctypes.windll.psapi.GetModuleBaseNameA(process, module, base_name, ctypes.sizeof(base_name))
            return base_name.value

    finally:
        # make sure to close it
        ctypes.windll.kernel32.CloseHandle(process)


# noinspection PyTypeChecker,PyCallingNonCallable
def get_pids(pid_process=(ctypes.c_long * 1024)()):
    """
    https://docs.microsoft.com/en-us/windows/desktop/api/psapi/nf-psapi-enumprocesses
    BOOL EnumProcesses(
        DWORD   *lpidProcess,
        DWORD   cb,
        LPDWORD lpcbNeeded
    );
    """
    cb = ctypes.c_long(ctypes.sizeof(pid_process))
    needed = ctypes.c_long()
    ctypes.windll.psapi.EnumProcesses(pid_process, cb, ctypes.byref(needed))
    return pid_process[:needed.value // ctypes.sizeof(ctypes.c_long)]


def find_process(name):
    for pid in get_pids():
        pname = get_process_name(pid)
        if pname and pname.startswith(name):
            return pid
