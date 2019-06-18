import platform
import sys
import threading
import tkinter as tk

import checker
import interop
import settings

NAME_MAPPING = [
    'NUL', 'SOH', 'STX', 'ETX', 'EOT',
    'ENQ', 'ACK', 'BEL',  'BS', 'TAB',
    'LF',   'VT',  'FF',  'CR',  'SO',
    'SI',  'DLE', 'DC1', 'DC2', 'DC3',
    'DC4', 'NAK', 'SYN', 'ETB', 'CAN',
    'EM',  'SUB', 'ESC',  'FS',  'GS',
    'RS',   'US', '???', 'RPG', 'APG',
    'END', 'STR', 'LFT',  'UP', 'RGT',
    'DWN'
]


class InputWindow(tk.Toplevel):
    def __init__(self, text, validator):
        super().__init__()
        self.validator = validator
        self.result = None

        self.hint_label = tk.Label(self, text=text)
        self.hint_label.pack(side=tk.TOP)

        self.input_entry = tk.Entry(self)
        self.input_entry.pack(side=tk.BOTTOM)
        self.input_entry.bind('<Key>', self.reset)
        self.input_entry.bind('<Return>', self.check)
        self.input_entry.focus()

    def reset(self, _event):
        self.input_entry.config(bg='white')

    def check(self, _event):
        text = self.input_entry.get()
        if self.validator(text):
            self.result = text
            self.destroy()
        else:
            self.input_entry.config(bg='red')

    @classmethod
    def run(cls, text, validator):
        window = InputWindow(text, validator)
        window.wait_window()
        return window.result


import logging
logging.basicConfig(level=logging.INFO)


class ConfigButton(tk.Button):
    def __init__(self, parent, text, row, column, command):
        super().__init__(parent, text=text, command=self.command)
        self.grid(row=row, column=column, sticky=tk.NSEW)
        self.real_command = command

    def command(self):
        self.config(state=tk.DISABLED)

        def run():
            try:
                self.real_command()
            except Exception as e:
                logging.exception('oops')
            finally:
                self.config(state=tk.NORMAL)

        threading.Thread(target=run).start()


# noinspection PyMethodMayBeStatic
class Application(tk.Tk):
    def __init__(self, chkr):
        super().__init__()

        self.checker = chkr
        self.title('POE Hacks')
        self.geometry('640x280')
        self.grid_columnconfigure(0, weight=1)
        self.grid_rowconfigure(1, weight=1)

        # Welcome, configure decoration, logout key
        self.sign_in_label = tk.Label(self, text='POE Hacks', font=('Arial', 16))
        self.sign_in_label.grid(row=0, column=0)

        self.config_deco_button = ConfigButton(self, '', 0, 1, self.config_deco)
        self.setting_deco()

        self.config_logout_button = ConfigButton(self, '', 0, 2, self.config_logout)
        self.setting_logout()

        self.toggle_checks_button = tk.Button(self, text='Toggle\nChecks', fg='green', command=self.toggle_checks)
        self.toggle_checks_button.grid(row=0, column=3)

        # Added actions
        self.action_list = tk.Listbox(self)
        self.action_list.grid(row=1, column=0, columnspan=4, sticky=tk.NSEW)
        self.action_list.bind('<BackSpace>', self.delete_action)
        self.action_list.bind('<Delete>', self.delete_action)
        for action in self.checker.settings.actions:
            self.action_list.insert(tk.END, action)

        # Add actions
        self.quit_button = tk.Button(self, text='Quit', command=self.destroy)
        self.quit_button.grid(row=2, column=0, sticky=tk.NSEW)

        self.add_logout_screen_button = ConfigButton(
            self,
            'Add auto\nlogout on\nscreen change',
            2, 1, self.config_logout_screen
        )
        self.add_key_screen_button = ConfigButton(
            self,
            'Add auto\nkey press on\nscreen change',
            2, 2, self.config_key_screen
        )
        self.add_key_key_button = ConfigButton(
            self,
            'Add auto\nkey press on\nkey press',
            2, 3, self.config_key_key
        )

        # Status bar
        self.status_label = tk.Label(self, bd=1, relief=tk.SUNKEN, anchor=tk.S)
        self.status_label.grid(row=3, column=0, columnspan=4, sticky=tk.EW)
        self.status()

    def status(self, text=None):
        if not text:
            text = 'Checks are running!'

        self.status_label.config(text=text)

    # SETTING DECORATION

    def config_deco(self):
        self.status('Please right-click on the first decoration in POE')
        p = interop.wait_mouse(2)
        c = interop.get_color(*p)
        self.checker.settings.decoration.p1 = p
        self.checker.settings.decoration.c1 = c

        self.status('Now right-click on the second decoration in POE')
        p = interop.wait_mouse(2)
        c = interop.get_color(*p)
        self.checker.settings.decoration.p2 = p
        self.checker.settings.decoration.c2 = c

        self.status()
        self.setting_deco()

    def setting_deco(self):
        point = self.checker.settings.decoration.p1
        self.config_deco_button.config(
            text='Configure\nDecoration',
            fg='red' if point == (0, 0) else 'green',
            state=tk.NORMAL
        )

    # SETTING LOGOUT

    def config_logout(self):
        self.status('Please press the key to use for logging out')
        self.checker.settings.logout_key = interop.wait_key()
        self.status()
        self.setting_logout()

    def setting_logout(self):
        key = self.checker.settings.logout_key
        self.config_logout_button.config(
            text=f'Logout\nKey ({key})' if key else 'Logout\nKey',
            fg='green' if key else 'red',
            state=tk.NORMAL
        )

    # TOGGLING CHECKS ON / OFF

    def toggle_checks(self):
        if self.checker.toggle():
            self.toggle_checks_button.config(fg='green')
        else:
            self.toggle_checks_button.config(fg='red')

    # ADDING LOGOUT ON SCREEN CHANGE

    def config_logout_screen(self):
        self.status('Right-click the life point to detect for logging out')
        point = interop.wait_mouse(2)
        color = interop.get_color(*point)
        self.add_action(settings.ScreenLogout(point, color))
        self.status()

    # ADDING KEY ON SCREEN CHANGE

    def config_key_screen(self):
        self.status('Press the key to auto use')
        key = interop.wait_key()
        self.status('Right-click the life/mana point to detect for using a key')
        point = interop.wait_mouse(2)
        color = interop.get_color(*point)
        self.status('Please fill in the delay dialog')
        delay = int(InputWindow.run('Enter flask delay (in ms)', lambda t: t.isdigit()) or 1000)
        self.add_action(settings.ScreenKey(point, color, key, delay))
        self.status()

    # ADDING KEY ON KEY PRESS

    def config_key_key(self):
        self.status('Press the key to detect for using another key')
        detect = interop.wait_key()
        self.status('Press the key to auto use')
        use = interop.wait_key()
        self.status('Please fill in the delay dialog')
        delay = int(InputWindow.run('Enter flask delay (in ms)', lambda t: t.isdigit()) or 1000)
        self.add_action(settings.KeyKey(detect, use, delay))
        self.status()

    # CREATING A NEW ACTION

    def add_action(self, action):
        self.checker.settings.actions.append(action)
        self.action_list.insert(tk.END, action)

    def delete_action(self, _event):
        index = self.action_list.curselection()
        if not index:
            return

        index = index[0]
        del self.checker.settings.actions[index]
        self.action_list.delete(index)

        if index == len(self.checker.settings.actions):
            index -= 1
        self.action_list.selection_set(index)


def main():
    if platform.architecture()[0].startswith('32'):
        # https://docs.microsoft.com/en-us/windows/desktop/api/psapi/nf-psapi-enumprocessmodules
        # If this function is called from a 32-bit application running on WOW64, it can only enumerate
        # the modules of a 32-bit process. If the process is a 64-bit process, this function fails and
        # the last error code is ERROR_PARTIAL_COPY (299).
        print('refusing to run with python 32 bits', file=sys.stderr)
        return 1

    if not interop.is_admin():
        interop.elevate(__file__)
        return -1

    with checker.Checker('poe.key') as c:
        Application(c).mainloop()


if __name__ == '__main__':
    quit(main())
