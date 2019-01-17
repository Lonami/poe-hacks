#include "point.h"

bool operator==(const Point& lhs, const Point& rhs) {
    return lhs.x == rhs.x
        && lhs.y == rhs.y;
}

bool operator!=(const Point& lhs, const Point& rhs) {
    return lhs.x != rhs.x
        && lhs.y != rhs.y;
}

std::ostream& operator<<(std::ostream& lhs, const Point& rhs) {
    lhs << rhs.x << ' ' << rhs.y;
    return lhs;
}

std::istream& operator>>(std::istream& lhs, Point& rhs) {
    lhs >> rhs.x >> rhs.y;
    return lhs;
}

