"""Early real-child diagnostic; no guest code is executed or acceptance reused."""
import json
import struct
import time
from pathlib import Path

import m3_runtime as runtime


def run():
    folder = runtime.ROOT / 'build/m3/P3-14' / ('host-preflight-' + str(time.time_ns()))
    folder.mkdir(parents=True)
    # Header admission is sufficient for this host-only fixture; never executed.
    image = folder / 'identity-only.elf'
    header = bytearray(64)
    header[:7] = b'\x7fELF\x02\x01\x01'
    struct.pack_into('<HH', header, 16, 2, 62)
    image.write_bytes(header)
    results = []
    for case, expected, code in [('success', 'complete', 0), ('nonzero', 'exited-nonzero', 7),
                                  ('missing-final', 'missing-final-record', 0), ('timeout', 'timeout', None)]:
        directory, report = runtime.run(runtime.PROFILE, image, runtime.digest(image), case)
        results.append(dict(path=str(directory), result=report))
        runtime.save(folder / 'results.json', results)
        runtime.require(report['outcome'] == expected and report['launched'] and report['cleanup'] == 'reaped'
                        and (code is None or report['exit_code'] == code), 'host preflight failed: ' + case)
    print('M3 host lifecycle preflight: ' + str(folder), flush=True)


if __name__ == '__main__':
    run()
