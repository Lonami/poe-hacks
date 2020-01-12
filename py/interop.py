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

# https://docs.microsoft.com/en-us/windows/desktop/inputdev/virtual-key-codes
# With some modifications
KEY_NAMES = [
    ('VK_NONE', 0x00, '(null)', '(null)'),
    ('VK_LBUTTON', 0x01, 'Left mouse button', 'LMB'),
    ('VK_RBUTTON', 0x02, 'Right mouse button', 'RMB'),
    ('VK_CANCEL', 0x03, 'Control-break processing', 'CANCEL'),
    ('VK_MBUTTON', 0x04, 'Middle mouse button (three-button mouse)', 'MMB'),
    ('VK_XBUTTON1', 0x05, 'X1 mouse button', 'X1MB'),
    ('VK_XBUTTON2', 0x06, 'X2 mouse button', 'X2MB'),
    ('VK_UNDEFINED', 0x07, 'Undefined', '(undef)'),
    ('VK_BACK', 0x08, 'BACKSPACE key', 'BACK'),
    ('VK_TAB', 0x09, 'TAB key', 'TAB'),
    ('VK_RESERVED', 0x0A, 'Reserved', '(res)'),
    ('VK_RESERVED', 0x0B, 'Reserved', '(res)'),
    ('VK_CLEAR', 0x0C, 'CLEAR key', ''),
    ('VK_RETURN', 0x0D, 'ENTER key', ''),
    ('VK_UNDEFINED', 0x0E, 'Undefined', '(undef)'),
    ('VK_UNDEFINED', 0x0F, 'Undefined', '(undef)'),
    ('VK_SHIFT', 0x10, 'SHIFT key', 'SHIFT'),
    ('VK_CONTROL', 0x11, 'CTRL key', 'CONTROL'),
    ('VK_MENU', 0x12, 'ALT key', 'MENU'),
    ('VK_PAUSE', 0x13, 'PAUSE key', 'PAUSE'),
    ('VK_CAPITAL', 0x14, 'CAPS LOCK key', 'CAPS'),
    ('VK_HANGUL', 0x15, 'IME Hangul mode (also Kana)', 'IME1'),
    ('VK_UNDEFINED', 0x16, 'Undefined', '(undef)'),
    ('VK_JUNJA', 0x17, 'IME Junja mode', 'IME2'),
    ('VK_FINAL', 0x18, 'IME final mode', 'IM3'),
    ('VK_KANJI', 0x19, 'IME Kanji mode (also Hanja)', 'IME4'),
    ('VK_UNDEFINED', 0x1A, 'Undefined', '(undef)'),
    ('VK_ESCAPE', 0x1B, 'ESC key', 'ESC'),
    ('VK_CONVERT', 0x1C, 'IME convert', 'IME5'),
    ('VK_NONCONVERT', 0x1D, 'IME nonconvert', 'IME6'),
    ('VK_ACCEPT', 0x1E, 'IME accept', 'IME7'),
    ('VK_MODECHANGE', 0x1F, 'IME mode change request', 'IME8'),
    ('VK_SPACE', 0x20, 'SPACEBAR', ' '),
    ('VK_PRIOR', 0x21, 'PAGE UP key', 'PG UP'),
    ('VK_NEXT', 0x22, 'PAGE DOWN key', 'PG DW'),
    ('VK_END', 0x23, 'END key', 'END'),
    ('VK_HOME', 0x24, 'HOME key', 'HOME'),
    ('VK_LEFT', 0x25, 'LEFT ARROW key', 'LEFT'),
    ('VK_UP', 0x26, 'UP ARROW key', 'UP'),
    ('VK_RIGHT', 0x27, 'RIGHT ARROW key', 'RIGHT'),
    ('VK_DOWN', 0x28, 'DOWN ARROW key', 'DOWN'),
    ('VK_SELECT', 0x29, 'SELECT key', 'SEL'),
    ('VK_PRINT', 0x2A, 'PRINT key', 'PRINT'),
    ('VK_EXECUTE', 0x2B, 'EXECUTE key', 'EXEC'),
    ('VK_SNAPSHOT', 0x2C, 'PRINT SCREEN key', 'PR SCR'),
    ('VK_INSERT', 0x2D, 'INS key', 'INS'),
    ('VK_DELETE', 0x2E, 'DEL key', 'DEL'),
    ('VK_HELP', 0x2F, 'HELP key', 'HELP'),
    ('VK_NUM', 0x30, '0 key', '0'),
    ('VK_NUM', 0x31, '1 key', '1'),
    ('VK_NUM', 0x32, '2 key', '2'),
    ('VK_NUM', 0x33, '3 key', '3'),
    ('VK_NUM', 0x34, '4 key', '4'),
    ('VK_NUM', 0x35, '5 key', '5'),
    ('VK_NUM', 0x36, '6 key', '6'),
    ('VK_NUM', 0x37, '7 key', '7'),
    ('VK_NUM', 0x38, '8 key', '8'),
    ('VK_NUM', 0x39, '9 key', '9'),
    ('VK_UNDEFINED', 0x3A, 'Undefined', '(undef)'),
    ('VK_UNDEFINED', 0x3B, 'Undefined', '(undef)'),
    ('VK_UNDEFINED', 0x3C, 'Undefined', '(undef)'),
    ('VK_UNDEFINED', 0x3D, 'Undefined', '(undef)'),
    ('VK_UNDEFINED', 0x3E, 'Undefined', '(undef)'),
    ('VK_UNDEFINED', 0x3F, 'Undefined', '(undef)'),
    ('VK_UNDEFINED', 0x40, 'Undefined', '(undef)'),
    ('VK_LETTER', 0x41, 'A key', 'A'),
    ('VK_LETTER', 0x42, 'B key', 'B'),
    ('VK_LETTER', 0x43, 'C key', 'C'),
    ('VK_LETTER', 0x44, 'D key', 'D'),
    ('VK_LETTER', 0x45, 'E key', 'E'),
    ('VK_LETTER', 0x46, 'F key', 'F'),
    ('VK_LETTER', 0x47, 'G key', 'G'),
    ('VK_LETTER', 0x48, 'H key', 'H'),
    ('VK_LETTER', 0x49, 'I key', 'I'),
    ('VK_LETTER', 0x4A, 'J key', 'J'),
    ('VK_LETTER', 0x4B, 'K key', 'K'),
    ('VK_LETTER', 0x4C, 'L key', 'L'),
    ('VK_LETTER', 0x4D, 'M key', 'M'),
    ('VK_LETTER', 0x4E, 'N key', 'N'),
    ('VK_LETTER', 0x4F, 'O key', 'O'),
    ('VK_LETTER', 0x50, 'P key', 'P'),
    ('VK_LETTER', 0x51, 'Q key', 'Q'),
    ('VK_LETTER', 0x52, 'R key', 'R'),
    ('VK_LETTER', 0x53, 'S key', 'S'),
    ('VK_LETTER', 0x54, 'T key', 'T'),
    ('VK_LETTER', 0x55, 'U key', 'U'),
    ('VK_LETTER', 0x56, 'V key', 'V'),
    ('VK_LETTER', 0x57, 'W key', 'W'),
    ('VK_LETTER', 0x58, 'X key', 'X'),
    ('VK_LETTER', 0x59, 'Y key', 'Y'),
    ('VK_LETTER', 0x5A, 'Z key', 'Z'),
    ('VK_LWIN', 0x5B, 'Left Windows key (Natural keyboard)', 'LWIN'),
    ('VK_RWIN', 0x5C, 'Right Windows key (Natural keyboard)', 'RWIN'),
    ('VK_APPS', 0x5D, 'Applications key (Natural keyboard)', 'APPS'),
    ('VK_RESERVED', 0x5E, 'Reserved', '(res)'),
    ('VK_SLEEP', 0x5F, 'Computer Sleep key', 'SLEEP'),
    ('VK_NUMPAD0', 0x60, 'Numeric keypad 0 key', 'NUM0'),
    ('VK_NUMPAD1', 0x61, 'Numeric keypad 1 key', 'NUM1'),
    ('VK_NUMPAD2', 0x62, 'Numeric keypad 2 key', 'NUM2'),
    ('VK_NUMPAD3', 0x63, 'Numeric keypad 3 key', 'NUM3'),
    ('VK_NUMPAD4', 0x64, 'Numeric keypad 4 key', 'NUM4'),
    ('VK_NUMPAD5', 0x65, 'Numeric keypad 5 key', 'NUM5'),
    ('VK_NUMPAD6', 0x66, 'Numeric keypad 6 key', 'NUM6'),
    ('VK_NUMPAD7', 0x67, 'Numeric keypad 7 key', 'NUM7'),
    ('VK_NUMPAD8', 0x68, 'Numeric keypad 8 key', 'NUM8'),
    ('VK_NUMPAD9', 0x69, 'Numeric keypad 9 key', 'NUM9'),
    ('VK_MULTIPLY', 0x6A, 'Multiply key', '*'),
    ('VK_ADD', 0x6B, 'Add key', '+'),
    ('VK_SEPARATOR', 0x6C, 'Separator key', 'SEP'),
    ('VK_SUBTRACT', 0x6D, 'Subtract key', '-'),
    ('VK_DECIMAL', 0x6E, 'Decimal key', 'DEC'),
    ('VK_DIVIDE', 0x6F, 'Divide key', '/'),
    ('VK_F1', 0x70, 'F1 key', 'F1'),
    ('VK_F2', 0x71, 'F2 key', 'F2'),
    ('VK_F3', 0x72, 'F3 key', 'F3'),
    ('VK_F4', 0x73, 'F4 key', 'F4'),
    ('VK_F5', 0x74, 'F5 key', 'F5'),
    ('VK_F6', 0x75, 'F6 key', 'F6'),
    ('VK_F7', 0x76, 'F7 key', 'F7'),
    ('VK_F8', 0x77, 'F8 key', 'F8'),
    ('VK_F9', 0x78, 'F9 key', 'F9'),
    ('VK_F10', 0x79, 'F10 key', 'F10'),
    ('VK_F11', 0x7A, 'F11 key', 'F11'),
    ('VK_F12', 0x7B, 'F12 key', 'F12'),
    ('VK_F13', 0x7C, 'F13 key', 'F13'),
    ('VK_F14', 0x7D, 'F14 key', 'F14'),
    ('VK_F15', 0x7E, 'F15 key', 'F15'),
    ('VK_F16', 0x7F, 'F16 key', 'F16'),
    ('VK_F17', 0x80, 'F17 key', 'F17'),
    ('VK_F18', 0x81, 'F18 key', 'F18'),
    ('VK_F19', 0x82, 'F19 key', 'F19'),
    ('VK_F20', 0x83, 'F20 key', 'F20'),
    ('VK_F21', 0x84, 'F21 key', 'F21'),
    ('VK_F22', 0x85, 'F22 key', 'F22'),
    ('VK_F23', 0x86, 'F23 key', 'F23'),
    ('VK_F24', 0x87, 'F24 key', 'F24'),
    ('VK_UNASSIGNED', 0x88, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0x89, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0x8A, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0x8B, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0x8C, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0x8D, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0x8E, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0x8F, 'Unassigned', '(unassign)'),
    ('VK_NUMLOCK', 0x90, 'NUM LOCK key', 'NUM LOCK'),
    ('VK_SCROLL', 0x91, 'SCROLL LOCK key', 'SCROLL LOCK'),
    ('VK_OEM', 0x92, 'OEM specific', ''),
    ('VK_OEM', 0x93, 'OEM specific', ''),
    ('VK_OEM', 0x94, 'OEM specific', ''),
    ('VK_OEM', 0x95, 'OEM specific', ''),
    ('VK_OEM', 0x96, 'OEM specific', ''),
    ('VK_UNASSIGNED', 0x97, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0x98, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0x99, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0x9A, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0x9B, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0x9C, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0x9D, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0x9E, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0x9F, 'Unassigned', '(unassign)'),
    ('VK_LSHIFT', 0xA0, 'Left SHIFT key', 'LSHIFT'),
    ('VK_RSHIFT', 0xA1, 'Right SHIFT key', 'RSHIFT'),
    ('VK_LCONTROL', 0xA2, 'Left CONTROL key', 'LCTRL'),
    ('VK_RCONTROL', 0xA3, 'Right CONTROL key', 'RCTRL'),
    ('VK_LMENU', 0xA4, 'Left MENU key', 'LMENU'),
    ('VK_RMENU', 0xA5, 'Right MENU key', 'RMENU'),
    ('VK_BROWSER_BACK', 0xA6, 'Browser Back key', 'BRW BCK'),
    ('VK_BROWSER_FORWARD', 0xA7, 'Browser Forward key', 'BRW FWD'),
    ('VK_BROWSER_REFRESH', 0xA8, 'Browser Refresh key', 'BRW REF'),
    ('VK_BROWSER_STOP', 0xA9, 'Browser Stop key', 'BRW STP'),
    ('VK_BROWSER_SEARCH', 0xAA, 'Browser Search key', 'BRW SRC'),
    ('VK_BROWSER_FAVORITES', 0xAB, 'Browser Favorites key', 'BRW FAV'),
    ('VK_BROWSER_HOME', 0xAC, 'Browser Start and Home key', 'BRW STR'),
    ('VK_VOLUME_MUTE', 0xAD, 'Volume Mute key', 'VOL MUTE'),
    ('VK_VOLUME_DOWN', 0xAE, 'Volume Down key', 'VOL DOWN'),
    ('VK_VOLUME_UP', 0xAF, 'Volume Up key', 'VOL UP'),
    ('VK_MEDIA_NEXT_TRACK', 0xB0, 'Next Track key', 'TRK NEXT'),
    ('VK_MEDIA_PREV_TRACK', 0xB1, 'Previous Track key', 'TRK PREV'),
    ('VK_MEDIA_STOP', 0xB2, 'Stop Media key', 'TRK STOP'),
    ('VK_MEDIA_PLAY_PAUSE', 0xB3, 'Play/Pause Media key', 'TRK PLAY'),
    ('VK_LAUNCH_MAIL', 0xB4, 'Start Mail key', 'MAIL'),
    ('VK_LAUNCH_MEDIA_SELECT', 0xB5, 'Select Media key', 'MEDIA'),
    ('VK_LAUNCH_APP1', 0xB6, 'Start Application 1 key', 'APP1'),
    ('VK_LAUNCH_APP2', 0xB7, 'Start Application 2 key', 'APP2'),
    ('VK_RESERVED', 0xB8, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xB9, 'Reserved', '(res)'),
    ('VK_OEM_1', 0xBA, 'Used for miscellaneous characters; it can vary by keyboard.<br> For the US standard keyboard, the \';:\' key', '`'),
    ('VK_OEM_PLUS', 0xBB, 'For any country/region, the \'+\' key', '+'),
    ('VK_OEM_COMMA', 0xBC, 'For any country/region, the \',\' key', ','),
    ('VK_OEM_MINUS', 0xBD, 'For any country/region, the \'-\' key', '-'),
    ('VK_OEM_PERIOD', 0xBE, 'For any country/region, the \'.\' key', '.'),
    ('VK_OEM_2', 0xBF, 'Used for miscellaneous characters; it can vary by keyboard.<br> For the US standard keyboard, the \'/?\' key', 'ç'),
    ('VK_OEM_3', 0xC0, 'Used for miscellaneous characters; it can vary by keyboard.\nFor the US standard keyboard, the \'`~\' key', ''),
    ('VK_RESERVED', 0xc1, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xc2, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xc3, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xc4, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xc5, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xc6, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xc7, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xc8, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xc9, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xca, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xcb, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xcc, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xcd, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xce, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xcf, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xd0, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xd1, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xd2, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xd3, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xd4, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xd5, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xd6, 'Reserved', '(res)'),
    ('VK_RESERVED', 0xd7, 'Reserved', '(res)'),
    ('VK_UNASSIGNED', 0xd8, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0xd9, 'Unassigned', '(unassign)'),
    ('VK_UNASSIGNED', 0xda, 'Unassigned', '(unassign)'),
    ('VK_OEM_4', 0xDB, 'Used for miscellaneous characters; it can vary by keyboard.\nFor the US standard keyboard, the \'[{\' key', '\''),
    ('VK_OEM_5', 0xDC, 'Used for miscellaneous characters; it can vary by keyboard.\nFor the US standard keyboard, the \'\|\' key', 'º'),
    ('VK_OEM_6', 0xDD, 'Used for miscellaneous characters; it can vary by keyboard.\nFor the US standard keyboard, the \']}\' key', '¡'),
    ('VK_OEM_7', 0xDE, 'Used for miscellaneous characters; it can vary by keyboard.\nFor the US standard keyboard, the \'single-quote/double-quote\' key', '´'),
    ('VK_OEM_8', 0xDF, 'Used for miscellaneous characters; it can vary by keyboard.', ''),
    ('VK_RESERVED', 0xE0, 'Reserved', '(res)'),
    ('VK_OEM', 0xE1, 'OEM specific', ''),
    ('VK_OEM_102', 0xE2, 'Either the angle bracket key or the backslash key on the RT 102-key keyboard', '<'),
    ('VK_OEM', 0xE3, 'OEM specific', ''),
    ('VK_OEM', 0xE4, 'OEM specific', ''),
    ('VK_PROCESSKEY', 0xE5, 'IME PROCESS key', 'PROC'),
    ('VK_OEM', 0xE6, 'OEM specific', ''),
    ('VK_PACKET', 0xE7, 'Used to pass Unicode characters as if they were keystrokes. The VK_PACKET key is the low word of a 32-bit Virtual Key value used for non-keyboard input methods. For more information, see Remark in <a href="https://msdn.microsoft.com/en-us/library/ms646271(v=VS.85).aspx" data-linktype="external"><strong>KEYBDINPUT</strong></a>, <a href="https://msdn.microsoft.com/en-us/library/ms646310(v=VS.85).aspx" data-linktype="external"><strong>SendInput</strong></a>, <a href="wm-keydown" data-linktype="relative-path"><strong>WM_KEYDOWN</strong></a>, and <a href="wm-keyup" data-linktype="relative-path"><strong>WM_KEYUP</strong></a>', ''),
    ('VK_UNASSIGNED', 0xE8, 'Unassigned', '(unassign)'),
    ('VK_OEM', 0xe9, 'OEM specific', ''),
    ('VK_OEM', 0xea, 'OEM specific', ''),
    ('VK_OEM', 0xeb, 'OEM specific', ''),
    ('VK_OEM', 0xec, 'OEM specific', ''),
    ('VK_OEM', 0xed, 'OEM specific', ''),
    ('VK_OEM', 0xee, 'OEM specific', ''),
    ('VK_OEM', 0xef, 'OEM specific', ''),
    ('VK_OEM', 0xf0, 'OEM specific', ''),
    ('VK_OEM', 0xf1, 'OEM specific', ''),
    ('VK_OEM', 0xf2, 'OEM specific', ''),
    ('VK_OEM', 0xf3, 'OEM specific', ''),
    ('VK_OEM', 0xf4, 'OEM specific', ''),
    ('VK_OEM', 0xf5, 'OEM specific', ''),
    ('VK_ATTN', 0xF6, 'Attn key', 'ATTN'),
    ('VK_CRSEL', 0xF7, 'CrSel key', 'CRSEL'),
    ('VK_EXSEL', 0xF8, 'ExSel key', 'EXSEL'),
    ('VK_EREOF', 0xF9, 'Erase EOF key', 'EREOF'),
    ('VK_PLAY', 0xFA, 'Play key', 'PLAY'),
    ('VK_ZOOM', 0xFB, 'Zoom key', 'ZOOM'),
    ('VK_NONAME', 0xFC, 'Reserved', '(res)'),
    ('VK_PA1', 0xFD, 'PA1 key', 'PA1'),
    ('VK_OEM_CLEAR', 0xFE, 'Clear key', 'CLEAR'),
    ('VK_NONE', 0xFF, '(null)', '(null)'),
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
    # TODO this program breaks because:
    # > After painting with a common DC, the ReleaseDC
    # > function must be called to release the DC
    # (https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdc)
    #
    # And we're not calling it anywhere which eventually breaks.
    # This is the same issue we found in the Rust rewrite but we
    # worked around that. The fix is to use a global here and only
    # fetch it once which should be more efficient regardless.
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
