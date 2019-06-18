import threading
import time

import interop
import settings


class Checker:
    def __init__(self, file):
        self.settings = settings.Settings.load(file)
        self.running = True
        self.checking = True
        self.thread = None

    def toggle(self):
        self.checking = not self.checking
        return self.checking

    def __enter__(self):
        self.running = True
        self.thread = threading.Thread(target=self.loop)
        self.thread.start()
        return self

    def __exit__(self, *args):
        self.running = False
        self.thread.join()
        self.thread = None
        self.settings.save()

    def loop(self, frequency=0.05):
        while self.running:
            time.sleep(frequency)
            if interop.is_down(self.settings.logout_key):
                interop.kill_connection(interop.find_process(b'PathOfExile'))

            if self.checking and self.settings.decoration.check():
                for action in self.settings.actions:
                    action.run()
