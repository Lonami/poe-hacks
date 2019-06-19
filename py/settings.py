import abc
import pickle
import time

import interop


def key_name(key):
    return interop.KEY_NAMES[key][-1] or '?'


class Decoration:
    def __init__(self, p1=(0, 0), c1=0, p2=(0, 0), c2=0):
        self.p1 = p1
        self.c1 = c1
        self.p2 = p2
        self.c2 = c2

    def __str__(self):
        return ' '.join(map(str, (
            *self.p1, self.c1,
            *self.p2, self.c2
        )))

    def check(self):
        return interop.get_color(*self.p1) == self.c1 \
               and interop.get_color(*self.p2) == self.c2


class Action(abc.ABC):
    def __init__(self, delay):
        self.delay = delay / 1000
        self.enabled = True
        self._last = 0

    def run(self):
        return time.time() - self._last > self.delay

    @abc.abstractmethod
    def __str__(self):
        pass


class KeyKey(Action):
    """
    Use a flask when you press a key.
    """
    def __init__(self, detect, use, delay):
        super().__init__(delay)
        self.detect = detect
        self.use = use

    def run(self):
        if super().run() and interop.is_down(self.detect):
            interop.press(self.use)
            self._last = time.time()

    def __str__(self):
        return \
            f'Use key {key_name(self.use)} ' \
            f'after detecting key {key_name(self.detect)} ' \
            f'every {self.delay:.2f}s'


class ScreenLogout(Action):
    """
    Logout on screen-point change.
    """
    def __init__(self, point, color):
        super().__init__(0)
        self.point = point
        self.color = color

    def run(self):
        if interop.get_color(*self.point) != self.color:
            interop.kill_connection(interop.find_process(b'PathOfExile'))

    def __str__(self):
        return 'Automatically logout on screen change'


class ScreenKey(Action):
    """
    Use flask on screen-point change.
    """
    def __init__(self, point, color, key, delay):
        super().__init__(delay)
        self.point = point
        self.color = color
        self.key = key

    def run(self):
        if super().run() and interop.get_color(*self.point) != self.color:
            interop.press(self.key)
            self._last = time.time()

    def __str__(self):
        what = 'life' if self.point[0] < 200 else 'mana'
        return \
            f'Use key {key_name(self.key)} ' \
            f'after {what} change ' \
            f'every {self.delay:.2f}s'


class Settings:
    def __init__(self, file):
        self.logout_key = 0
        self.decoration = Decoration()
        self.actions = []
        self.file = file

    @classmethod
    def load(cls, file):
        try:
            with open(file, 'rb') as fd:
                return pickle.load(fd)
        except OSError:
            return cls(file)

    def save(self):
        with open(self.file, 'wb') as fd:
            pickle.dump(self, fd)
