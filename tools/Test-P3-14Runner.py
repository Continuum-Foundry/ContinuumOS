"""Exercise real child-tree timeout cleanup and source-lock additions."""
import ctypes
import contextlib
import io
from ctypes import wintypes
import json
from pathlib import Path
import subprocess
import sys
import time
import unittest

import p3_acceptance as gate


class RunnerTests(unittest.TestCase):
    def scratch(self):
        folder = gate.ROOT / 'work' / ('p314-runner-test-' + str(time.time_ns()))
        folder.mkdir()
        return folder

    def test_real_success(self):
        folder = self.scratch()
        result = gate.command(folder, 'success', [sys.executable, '-c', 'print(42)'], 10)
        self.assertEqual(result['exit_code'], 0)
        self.assertTrue(result['cleanup_reaped'])

    def test_nonzero_is_failure(self):
        folder = self.scratch()
        with self.assertRaisesRegex(gate.CommandFailure, 'failed') as caught:
            gate.command(folder, 'bad-exit', [sys.executable, '-c', 'raise SystemExit(7)'], 10)
        result = gate.read(folder / 'bad-exit-command.json')
        self.assertEqual(caught.exception.result, result)
        self.assertEqual(result['exit_code'], 7)
        self.assertFalse(result['timed_out'])

    def test_progress_is_relayed_before_child_exit(self):
        folder = self.scratch()
        ack = folder / 'progress-relayed'
        class Sink(io.StringIO):
            def write(self, value):
                if 'M3 stage child-ready' in value:
                    ack.write_text('seen')
                return super().write(value)
        script = ('import time; from pathlib import Path; '
                  'print("M3 stage child-ready",flush=True); '
                  f'p=Path({str(ack)!r}); deadline=time.monotonic()+3\n'
                  'while not p.exists() and time.monotonic()<deadline: time.sleep(.02)\n'
                  'raise SystemExit(0 if p.exists() else 7)')
        with contextlib.redirect_stdout(Sink()):
            result = gate.command(folder, 'live-progress', [sys.executable, '-c', script], 5)
        self.assertTrue(ack.exists())
        self.assertLessEqual(result['started_utc'], result['finished_utc'])

    def test_timeout_reaps_descendant(self):
        folder = self.scratch()
        pid_path = folder / 'child.pid'
        script = ('import subprocess,sys,time; from pathlib import Path; '
                  'p=subprocess.Popen([sys.executable,"-c","import time; time.sleep(60)"],creationflags=0x08000000); '
                  f'Path({str(pid_path)!r}).write_text(str(p.pid)); time.sleep(60)')
        with self.assertRaisesRegex(ValueError, 'failed'):
            gate.command(folder, 'timeout', [sys.executable, '-c', script], 2)
        result = gate.read(folder / 'timeout-command.json')
        self.assertTrue(result['timed_out'] and result['cleanup_reaped'])
        pid = int(pid_path.read_text())
        api = ctypes.WinDLL('kernel32', use_last_error=True)
        api.OpenProcess.argtypes = (wintypes.DWORD, wintypes.BOOL, wintypes.DWORD)
        api.OpenProcess.restype = wintypes.HANDLE
        api.WaitForSingleObject.argtypes = (wintypes.HANDLE, wintypes.DWORD)
        api.CloseHandle.argtypes = (wintypes.HANDLE,)
        handle = api.OpenProcess(0x100000, False, pid)
        if handle:
            try:
                self.assertEqual(api.WaitForSingleObject(handle, 5000), 0, 'descendant survived timeout')
            finally:
                api.CloseHandle(handle)

    def test_source_lock_tracks_untracked_implementation(self):
        self.assertIn('tools/p3_acceptance.py', gate.source_lock())
        self.assertIn('config/p3-14-acceptance.json', gate.source_lock())

    def test_path_escape_rejected(self):
        with self.assertRaisesRegex(ValueError, 'outside workspace'):
            gate.local(gate.ROOT.parent / 'outside')

    def test_matrix_has_exact_story_rows(self):
        self.assertEqual(gate.read(gate.CONFIG)['required_matrix'], list(gate.MATRIX))
        self.assertEqual(len(gate.MATRIX), 23)


if __name__ == '__main__':
    unittest.main()
