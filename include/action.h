#pragma once

#include <iostream>
#include <string>

#include "point.h"
#include "color.h"

struct Action {
    // is this action enabled
    bool enabled;

    // the flask to use in this action, 0 = logout
    unsigned int flask;

    // the delay between spamming the flask, in ms
    unsigned int delay;
    
    // the skill that will trigger the flask too, 0 = point/color
    unsigned int skill;
    
    // the point and the color it should be to not trigger flask
    Point point;
    Color color;
    
    // description of this action
    std::string desc;

    // constructor
    Action();

    // checks whether the action should be executed
    bool check();
};

std::ostream& operator<<(std::ostream& lhs, const Action& rhs);
std::istream& operator>>(std::istream& lhs, Action& rhs);
