#include "settings.h"

#include <vector>
#include <cstdio>
#include <fstream>
#include <conio.h>

#include "decor.h"
#include "input.h"

namespace settings {

int width = 0;
int height = 0;
int logout_key = 0;
Decor decor;
std::vector<Action> actions  {};

bool load() {
    screen::size(width, height);

    decor.p1.x = (int)(width * DECO_X0);
    decor.p1.y = (int)(height * DECO_Y0);
    decor.c1 = screen::get(decor.p1);

    decor.p2.x = (int)(width * DECO_X1);
    decor.p2.y = (int)(height * DECO_Y1);
    decor.c2 = screen::get(decor.p2);

    actions = {};
    std::ifstream fin("poe.key");
    if (fin) {
        char key;
        float percent;
        int len;
        fin >> key
            >> percent
            >> len;

        // key should be uppercase
        if ('a' <= key && key <= 'z') {
            key += 'A' - 'a';
        }
        logout_key = (int)(key == '-' ? 0xBD : key); // special-cased -

        // figure out location with percent
        // TODO reuse this code and uppercase check code
        if (percent < 0) {
            fprintf(stderr, "percent too low (%f), using 0%%\n", percent);
            percent = 0;
        } else if (percent > 100) {
            fprintf(stderr, "percent too high (%f), using 100%%\n", percent);
            percent = 1;
        } else {
            percent /= 100;
        }

        printf("loaded logout key %c, autokick at %.1f%%\n", key, percent * 100);

        Action action;
        action.flask = 0;
        action.delay = 0;
        action.set_point(percent, false);
        actions.push_back(action); // by copy

        for (int i = 0; i < len; ++i) {
            fin >> action;
            actions.push_back(action);
        }

        fin.close();
        return true;
    } else {
        return false;
    }
}
}
