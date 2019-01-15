#pragma once

#include <Windows.h>

using InputCb = void (*)(int key, bool down);

// get the primary screen size
void getscreensize(int &w, int &h);

// set the mouse position
bool setmouse(int x, int y);

// get the mouse position, true on success
bool getmouse(int &x, int &y);

// clicks with button, 1 left, 2 right; default 1
void click();
void click(int button);

// scrolls the mouse n times, positive forward (up), negative backward (down)
void scroll(int amount);

// types the given text
void type(const char *text);

// presses the given key, and how (0 tap, 1 down, 2 up)
void press(WORD vk, char action);

// is the given key down?
bool isdown(char key);

// steps the input state to update it
void stepinput();

// sets the input callback, for possible keys see:
// https://docs.microsoft.com/en-gb/windows/desktop/inputdev/virtual-key-codes
void setinputcb(InputCb cb);
