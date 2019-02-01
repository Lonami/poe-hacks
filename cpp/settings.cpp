#include "settings.h"

#include <vector>
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

void plugins() {
    unsigned char ch1, ch2;
    int index = 0;

    cmd::cls();
    for (auto&& action: actions) {
        printf("[%c] %s, flask %c with delay %d, ",
               action.enabled ? 'x' : ' ',
               action.desc.c_str(),
               action.flask,
               action.delay);

        if (action.skill) {
            printf("skill %c\n", action.skill);
        } else if (action.point.x < width / 2) {
            printf("life point\n");
        } else {
            printf("mana point\n");
        }
    }
    printf(" <  save and exit (e/esc)\n");

    cmd::set(1, index);
    while (true) {
        ch1 = _getch();
        ch2 = _getch();
        switch (ch1) {
        case VK_RETURN:
        case VK_SPACE:
            if (index != actions.size()) {
                actions[index].enabled = !actions[index].enabled;
                cmd::set(1, index);
                putchar(actions[index].enabled ? 'x' : ' ');
                cmd::set(1, index);
            } else {
                return;
            }
            break;
        case VK_BACK:
        case VK_DELETE:
        case 'D':
            actions.erase(actions.begin() + index);
            if (!actions.empty() && index == actions.size()) {
                --index;
                cmd::set(0, index);
            }
            break;
        case VK_ESCAPE:
        case 'E':
            return;
        case 0xe0:
            switch (ch2) {
            case 72: // up
                index = (index > 0 ? index - 1 : actions.size());
                cmd::set(1, index);
                break;
            case 80: // down
                index = (index < actions.size() ? index + 1 : 0);
                cmd::set(1, index);
                break;
            }
            break;
        }
    }
}

void add_action(int index) {
    Action action;
    action.enabled = true;
    action.last_use = 0;

    if (index == 2) {
        printf("automatically logout on screen point change\n");
    } else if (index == 3) {
        printf("automatically use flask on screen point change\n");
    } else {
        printf("automatically use flask on using skill\n");
    }

    printf("enter a description for this point:\n");
    std::getline(std::cin, action.desc);

    if (index == 2) {
        action.flask = 0;
        action.delay = 0;
    } else {
        printf("press the key with the flask: ");
        fflush(stdout);
        action.flask = VkKeyScanA(_getch()) & 0xff;
        _getch(); // 0
        printf("%c\n", action.flask);

        printf("enter the delay in ms for using the flask: ");
        std::cin >> action.delay;
    }

    if (index == 4) {
        printf("press the key or click with the skill: ");
        fflush(stdout);
        action.skill = input::wait();
    } else {
        printf("right click the life/mana point to detect...");
        fflush(stdout);
        input::wait(VK_RBUTTON);
        action.skill = 0;
        action.point = mouse::get();
        action.color = screen::get(action.point);
        _getch();
        _getch();
    }

    actions.push_back(action);
}

void menu() {
    // getch is either char + null or char and char
    unsigned char ch1, ch2;
    int index = 0;

    std::ofstream fout;
    bool dirty = true;

    while (true) {
        if (dirty) {
            cmd::cls();
            printf("[ ] change decoration\n");
            printf("[ ] set logout key (current: %c)\n", logout_key);
            printf("[ ] add autologout on screen point change\n");
            printf("[ ] add autoflask on screen point change\n");
            printf("[ ] add autoflask on skill use\n");
            printf("[ ] view, enable, disable and delete actions\n");
            printf(" <  exit config, run program\n");
            printf("(use arrow keys to move)");
            fflush(stdout);
            save();
        }
        cmd::set(1, index);

        ch1 = _getch();
        ch2 = _getch();

        dirty = ch1 != 0xe0;
        if (dirty) {
            cmd::cls();
        }

        switch (ch1) {
        case '1':
        case '2':
        case '3':
        case '4':
        case '5':
        case '6':
        case '7':
            index = ch1 - '1';
            // fallthrough
        case VK_SPACE:
        case VK_RETURN:
            switch (index) {
            case 0: // decoration
                printf("decoration is used to detect when you're playing\n");
                printf("right click on two points that won't change in-game");
                fflush(stdout);
                decor.grab(VK_RBUTTON);
                break;
            case 1: // logout key
                printf("logout key is used when you need to manually quick dc\n");
                printf("press the key to use: ");
                fflush(stdout);
                logout_key = VkKeyScanA(_getch()) & 0xff;
                _getch(); // 0
                break;
            case 2: // add action (logout 2, point 3, skill 4)
            case 3: // all these are actions and share common paths
            case 4:
                add_action(index);
                break;
            case 5: // modify actions
                plugins();
                break;
            case 6: // exit config
                return;
            }
            break;

        case VK_ESCAPE:
        case 'E':
            return;

        case 0xe0:
            switch (ch2) {
            case 72: // up
                index = (index > 0 ? index - 1 : 6);
                cmd::set(1, index);
                break;
            case 80: // down
                index = (index < 6 ? index + 1 : 0);
                cmd::set(1, index);
                break;
            }
            break;
        }
    }
}

bool load() {
    int len;
    Action action;

    actions = {};
    screen::size(width, height);
    std::ifstream fin("poe.key");
    if (fin) {
        fin >> logout_key
            >> decor
            >> len;

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

void save() {
    std::ofstream fout("poe.key");
    
    // key that when hit will logout
    extern int logout_key;
    
    // decoration to check to make sure we're in game
    extern Decor decor;

    // actions loaded by the settings and used
    extern std::vector<Action> actions;

    
    fout << logout_key << '\n'
         << decor << '\n'
         << actions.size() << '\n';

    for (auto&& action: actions) {
        fout << action << '\n';
    }

    fout.close();
}
}
