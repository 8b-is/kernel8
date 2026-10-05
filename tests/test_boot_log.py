import importlib.util
from pathlib import Path
import unittest
spec = importlib.util.spec_from_file_location("verify_boot", Path(__file__).resolve().parents[1] / "scripts/verify_boot.py")
boot = importlib.util.module_from_spec(spec)
spec.loader.exec_module(boot)
GOOD = "\n".join(boot.REQUIRED) + "\n" + "." * 20 + "counted 20 timer quanta — discrete, not interpolated.\nfree-running now, one '.' per quantum:\n...\n"

class BootLogTests(unittest.TestCase):
    def test_valid_log(self):
        boot.validate(GOOD)

    def test_timer_can_advance_past_threshold(self):
        boot.validate(GOOD.replace("counted 20", "counted 21"))

    def test_corrupt_logs_rejected(self):
        cases = {
            "sequential": GOOD.replace("task b: 1\ntask a: 2", "task a: 2\ntask b: 1"),
            "heap": GOOD.replace("sum = 124750", "sum = 0"),
            "paging": GOOD.replace("read back 0xC0FFEE", "read back 0x0"),
            "panic": GOOD + "KERNEL PANIC: test",
            "reboot": GOOD + GOOD,
            "missing": GOOD.replace("task b: 2\n", ""),
            "duplicate": GOOD.replace("task a: 1\n", "task a: 1\ntask a: 1\n"),
            "timer": GOOD.replace("counted 20", "counted 19"),
        }
        for name, text in cases.items():
            with self.subTest(name=name), self.assertRaises(ValueError):
                boot.validate(text)
