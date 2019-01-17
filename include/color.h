#pragma once

#include <iostream>

// color union to support field-by-field access to/from COLORREF
// https://docs.microsoft.com/en-gb/windows/desktop/gdi/colorref
union Color
{
    unsigned long zbgr;
    struct { unsigned char b, g, r, a; };
};

bool operator==(const Color& lhs, const Color& rhs);
bool operator!=(const Color& lhs, const Color& rhs);
std::ostream& operator<<(std::ostream& lhs, const Color& rhs);
std::istream& operator>>(std::istream& lhs, Color& rhs);
