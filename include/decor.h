#pragma once

#include "point.h"
#include "color.h"

struct Decor {
    // the point and the color it should be to not trigger flask
    Point p1, p2;
    Color c1, c2;

    // checks whether the decoration is valid (in-game)
    bool check();
};
