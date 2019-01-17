#include "color.h"

bool operator==(const Color& lhs, const Color& rhs) {
    return lhs.zbgr == rhs.zbgr;
}

bool operator!=(const Color& lhs, const Color& rhs) {
    return lhs.zbgr != rhs.zbgr;
}

std::ostream& operator<<(std::ostream& lhs, const Color& rhs) {
    lhs << rhs.zbgr;
    return lhs;
}

std::istream& operator>>(std::istream& lhs, Color& rhs) {
    lhs >> rhs.zbgr;
    return lhs;
}
