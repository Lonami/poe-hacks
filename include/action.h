#pragma once

#include <iostream>
#include <string>

#include "point.h"
#include "color.h"

struct Action {
    // the flask to use in this action
    unsigned int flask;

    // last use of the flask, not saved
    unsigned int last_use;

    // the delay between spamming the flask, in ms
    unsigned int delay;

    // the point and the color it should be to not trigger flask
    Point point;
    Color color;

    // confgigures the point and color (for life or mana)
    void set_point(float percent, bool mana);

    // checks whether the action should be executed
    bool check();
};

std::istream& operator>>(std::istream& lhs, Action& rhs);
