#include "input.h"

#include <cstring>
#include <Wingdi.h>

InputCb inputcb = nullptr;

// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-getdesktopwindow
void getscreensize(int &w, int &h) {
    RECT desktop;
    const HWND hDesktop = GetDesktopWindow();
    GetWindowRect(hDesktop, &desktop);
    w = desktop.right;
    h = desktop.bottom;
}

// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-setcursorpos
bool setmouse(int x, int y) {
    return SetCursorPos(x, y);
}

bool setmouse(Point p) {
    return SetCursorPos(p.x, p.y);
}

// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-getcursorpos
Point getmouse() {
    POINT point;
    if (GetCursorPos(&point)) {
        return Point { point.x, point.y };
    } else {
        return Point { -1, -1 };
    }
}

// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-sendinput
void click() {
    click(1);
}

void click(int button) {
    MOUSEINPUT mi = {
        0, 0,                                                  // dx, dy
        static_cast<DWORD>(button == 2 ? XBUTTON2 : XBUTTON1), // button
        MOUSEEVENTF_LEFTDOWN,                                  // action
        0                                                      // time
    };
    INPUT input = { INPUT_MOUSE, mi };
    SendInput(1, &input, sizeof(input));
    input.mi.dwFlags = MOUSEEVENTF_LEFTUP;
    SendInput(1, &input, sizeof(input));
}

void scroll(DWORD amount) {
    MOUSEINPUT mi = {
        0, 0,              // dx, dy
        120 * amount,      // amount
        MOUSEEVENTF_WHEEL, // action
        0                  // time
    };
    INPUT input = { INPUT_MOUSE, mi };
    SendInput(1, &input, sizeof(input));
}

// helper method to send input depending on the modifier
// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-vkkeyscana#return-value
// https://docs.microsoft.com/en-gb/windows/desktop/inputdev/virtual-key-codes
void _typemod(INPUT &input, const unsigned char mod) {
    if (mod & 1) {
        input.ki.wVk = VK_SHIFT;
        SendInput(1, &input, sizeof(input));
    }
    if (mod & 2) {
        input.ki.wVk = VK_CONTROL;
        SendInput(1, &input, sizeof(input));
    }
    if (mod & 4) {
        input.ki.wVk = VK_MENU;
        SendInput(1, &input, sizeof(input));
    }
}

void type(const char *text) {
    INPUT input = { INPUT_KEYBOARD };
    input.ki = {
        0, 0, 0, 0, 0 // vk, scan code, flags, time, extra
    };
    while (*text) {
        unsigned short res = VkKeyScanA(*text++);
        unsigned char mod = res >> 8;
        unsigned char key = res & 0xff;

        // down
        input.ki.dwFlags = 0;
        _typemod(input, mod);
        input.ki.wVk = key;
        SendInput(1, &input, sizeof(input));

        // up
        input.ki.dwFlags = KEYEVENTF_KEYUP;
        SendInput(1, &input, sizeof(input));
        _typemod(input, mod);
    }
}

void press(WORD vk, char action) {
    INPUT input = { INPUT_KEYBOARD };
    input.ki = { vk, 0, 0, 0, 0 };
    switch (action) {
    case 0:
        SendInput(1, &input, sizeof(input));
        input.ki.dwFlags = KEYEVENTF_KEYUP;
        SendInput(1, &input, sizeof(input));
        break;
    case 1:
        SendInput(1, &input, sizeof(input));
        break;
    case 2:
        input.ki.dwFlags = KEYEVENTF_KEYUP;
        SendInput(1, &input, sizeof(input));
        break;
    };
}

bool isdown(char key) {
    return (GetKeyState(VkKeyScanA(key) & 0xff) & 0x80) != 0;
}

void stepinput() {
    static unsigned char kbd1[256]; // 1 = down, 0 = up
    static unsigned char kbd2[256];
    static unsigned char *before = kbd1;
    unsigned char *after = before == kbd1 ? kbd2 : kbd1;

    GetKeyState(0); // for some reason GetKeyboardState only works w/ this
    GetKeyboardState(after);
    
    if (inputcb != nullptr) {
        for (int i = 0; i < 256; ++i) {
            after[i] = after[i] & 0x80 ? 1 : 0;
            if (before[i] != after[i]) {
                inputcb(i, after[i]);
            }
        }
    }

    before = after;
}

void waitpress(int key) {
    while ((GetKeyState(key) & 0x80) == 0) Sleep(10);
    while ((GetKeyState(key) & 0x80) != 0) Sleep(10);
}

int waitinput() {
    int key;
    bool checking = true;
    unsigned char kbd1[256];
    unsigned char kbd2[256];

    GetKeyState(0);
    GetKeyboardState(kbd1);
    while (checking) {
        Sleep(10);
        GetKeyState(0);
        GetKeyboardState(kbd2);
        for (key = 0; key < 256; ++key) {
            if (kbd1[key] != kbd2[key]) {
                checking = false;
                break;
            }
        }
    }
    while ((GetKeyState(key) & 0x80) != 0) Sleep(10);
    return key;
}

void setinputcb(InputCb cb) {
    inputcb = cb;
}

Color getpixel(Point p) {
    static HDC dc = GetDC(NULL);
    return { GetPixel(dc, p.x, p.y) };
}
