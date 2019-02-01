#pragma once

#include <iostream>
#include <string>

#include "point.h"
#include "color.h"

struct Decor {
    // the point and the color it should be to not trigger flask
    Point p1, p2;
    Color c1, c2;

    // grab points from screen
    void grab(int wait_key);
    
    // checks whether the decoration is valid (in-game)
    bool check();
};

std::ostream& operator<<(std::ostream& lhs, const Decor& rhs);
std::istream& operator>>(std::istream& lhs, Decor& rhs);
