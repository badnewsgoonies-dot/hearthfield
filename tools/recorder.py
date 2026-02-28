"""
Hearthfield Screenshot Recorder
A small always-on-top GUI that captures the game window at ~5 FPS.
Usage: python tools/recorder.py
"""

import os
import time
import threading
from datetime import datetime
from tkinter import Tk, Frame, Label, Button, Entry, filedialog, StringVar, IntVar, BOTH, LEFT, RIGHT, X, W, E

import mss
from PIL import Image

# Try win32gui for window-specific capture; fall back to full screen
try:
    import win32gui
    HAS_WIN32 = True
except ImportError:
    HAS_WIN32 = False


def find_window_rect(title_substring="Hearthfield"):
    """Find a window whose title contains the substring and return its rect."""
    if not HAS_WIN32:
        return None
    result = []

    def enum_cb(hwnd, _):
        if win32gui.IsWindowVisible(hwnd):
            text = win32gui.GetWindowText(hwnd)
            if title_substring.lower() in text.lower():
                rect = win32gui.GetWindowRect(hwnd)
                result.append(rect)

    win32gui.EnumWindows(enum_cb, None)
    return result[0] if result else None


class Recorder:
    def __init__(self):
        self.root = Tk()
        self.root.title("Recorder")
        self.root.attributes("-topmost", True)
        self.root.resizable(False, False)

        self.recording = False
        self.thread = None
        self.frame_count = 0
        self.output_dir = StringVar(value=os.path.join(os.path.expanduser("~"), "Desktop", "recordings"))
        self.interval_ms = IntVar(value=200)  # ~5 FPS

        self._build_ui()

    def _build_ui(self):
        pad = dict(padx=6, pady=3)

        # -- Output folder row --
        row0 = Frame(self.root)
        row0.pack(fill=X, **pad)
        Label(row0, text="Folder:").pack(side=LEFT)
        Entry(row0, textvariable=self.output_dir, width=32).pack(side=LEFT, fill=X, expand=True, padx=(4, 4))
        Button(row0, text="Browse", command=self._browse).pack(side=RIGHT)

        # -- Interval row --
        row1 = Frame(self.root)
        row1.pack(fill=X, **pad)
        Label(row1, text="Interval (ms):").pack(side=LEFT)
        Entry(row1, textvariable=self.interval_ms, width=6).pack(side=LEFT, padx=(4, 0))
        Label(row1, text="(200 = ~5 fps)").pack(side=LEFT, padx=(8, 0))

        # -- Controls row --
        row2 = Frame(self.root)
        row2.pack(fill=X, **pad)

        self.btn = Button(row2, text="  REC  ", bg="#cc3333", fg="white",
                          font=("Consolas", 12, "bold"), command=self._toggle)
        self.btn.pack(side=LEFT)

        self.status_label = Label(row2, text="Stopped", font=("Consolas", 10))
        self.status_label.pack(side=LEFT, padx=(12, 0))

        self.count_label = Label(row2, text="0 frames", font=("Consolas", 10))
        self.count_label.pack(side=RIGHT)

    def _browse(self):
        d = filedialog.askdirectory(initialdir=self.output_dir.get())
        if d:
            self.output_dir.set(d)

    def _toggle(self):
        if self.recording:
            self._stop()
        else:
            self._start()

    def _start(self):
        # Create timestamped subfolder
        base = self.output_dir.get()
        stamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        self.session_dir = os.path.join(base, f"session_{stamp}")
        os.makedirs(self.session_dir, exist_ok=True)

        self.frame_count = 0
        self.recording = True
        self.btn.config(text="  STOP  ", bg="#336633")
        self.status_label.config(text="Recording...")

        self.thread = threading.Thread(target=self._capture_loop, daemon=True)
        self.thread.start()

    def _stop(self):
        self.recording = False
        self.btn.config(text="  REC  ", bg="#cc3333")
        self.status_label.config(text=f"Saved to {os.path.basename(self.session_dir)}")

    def _capture_loop(self):
        interval = max(self.interval_ms.get(), 50) / 1000.0

        with mss.mss() as sct:
            while self.recording:
                t0 = time.perf_counter()

                # Try to find the game window
                rect = find_window_rect("Hearthfield")
                if rect:
                    left, top, right, bottom = rect
                    monitor = {"left": left, "top": top,
                               "width": right - left, "height": bottom - top}
                else:
                    # Fallback: primary monitor
                    monitor = sct.monitors[1]

                shot = sct.grab(monitor)
                img = Image.frombytes("RGB", shot.size, shot.bgra, "raw", "BGRX")

                path = os.path.join(self.session_dir, f"frame_{self.frame_count:06d}.png")
                img.save(path, "PNG")
                self.frame_count += 1

                # Update UI from thread (schedule on main thread)
                self.root.after(0, self._update_count)

                elapsed = time.perf_counter() - t0
                sleep_time = interval - elapsed
                if sleep_time > 0:
                    time.sleep(sleep_time)

    def _update_count(self):
        self.count_label.config(text=f"{self.frame_count} frames")

    def run(self):
        self.root.mainloop()


if __name__ == "__main__":
    Recorder().run()
