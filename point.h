#pragma once

#include <iostream>

struct Point
{
    int x, y;
};

bool operator==(const Point& lhs, const Point& rhs);
bool operator!=(const Point& lhs, const Point& rhs);
std::ostream& operator<<(std::ostream& lhs, const Point& rhs);
std::istream& operator>>(std::istream& lhs, Point& rhs);
