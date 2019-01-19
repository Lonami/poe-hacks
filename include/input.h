#pragma once

#include <iostream>
#include <Windows.h>

#include "color.h"
#include "point.h"

using InputCb = void (*)(int key, bool down);

namespace screen {
    // get the primary screen size
    void size(int &w, int &h);

    // get a pixel onscreen
    Color get(Point p);

    // sticks the window up in screen
    bool stick();

    // unsticks the window from screen
    bool unstick();
}

namespace mouse {
    // get the mouse position
    Point get();

    // set the mouse position
    bool set(int x, int y);
    bool set(Point p);

    // clicks with button, 1 left, 2 right; default 1
    void click();
    void click(int button);

    // scrolls the mouse n times, positive forward (up), negative backward (down)
    void scroll(int amount);
}

namespace kbd {
    // types the given text
    void type(const char *text);

    // presses the given key
    void tap(WORD vk);
    void hold(WORD vk);
    void release(WORD vk);
    void press(WORD vk, bool down);

    // is the given key down?
    bool down(WORD vk);

    // is the given key pressed? (down and up)
    // NOTE the key should not change! it will
    // detect down and then up on any key.
    // NOTE it should be used in a loop until pressed
    bool pressed(WORD vk);
}

namespace input {
    // sets the input callback, for possible keys see:
    // https://docs.microsoft.com/en-gb/windows/desktop/inputdev/virtual-key-codes
    void setcb(InputCb cb);

    // steps the input state to update it
    void step();

    // wait until the given key is pressed (down and released)
    void wait(WORD vk);

    // wait until any key state changes and then is released, and return which
    WORD wait();
}
