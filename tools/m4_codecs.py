"""Host-only, explicitly experimental desktop byte codecs. No permanent IDs."""
import struct

HEADER = struct.Struct('<4sHHIHHQQQQQII')
BODY = {
    'surface': struct.Struct('<QQIIIIQIIIIQ'),
    'input': struct.Struct('<QQQIIiiII'),
    'semantic': struct.Struct('<QQQiiIIIIII'),
    'output': struct.Struct('<QQIIIIii'),
}
DEV = struct.Struct('<4sHHIIQHHI')


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def body_check(family, values, label=b''):
    require(family in BODY, 'unknown endpoint family')
    if family == 'surface':
        lease, scene, w, h, stride, fmt, size, x, y, dw, dh, generation = values
        require(lease > 0 and scene > 0 and generation > 0, 'zero surface identity')
        require(0 < w <= 1920 and 0 < h <= 1080 and fmt == 1, 'surface dimensions/format')
        require(w * 4 <= stride <= 8192 and stride % 4 == 0 and size == stride * h <= 8847360, 'surface backing')
        require(x + dw <= w and y + dh <= h, 'damage outside surface')
    elif family == 'input':
        stamp, scene, focus, device, kind, x, y, code, mods = values
        require(min(stamp, scene, focus, device) > 0 and 1 <= kind <= 3, 'input identity/kind')
        require(code <= 65535 and mods & ~255 == 0, 'input code/modifier mask')
    elif family == 'semantic':
        node, parent, scene, x, y, w, h, role, state, actions, length = values
        require(node > 0 and node != parent and scene > 0, 'semantic identity')
        require(w <= 1920 * 65536 and h <= 1080 * 65536 and 1 <= role <= 8, 'semantic bounds/role')
        require(state & ~65535 == actions & ~65535 == 0, 'semantic unsupported bits')
        require(length == len(label) <= 256 and b'\0' not in label, 'semantic label bounds')
        try:
            label.decode('utf-8', errors='strict')
        except UnicodeError as error:
            raise ValueError('semantic UTF-8') from error
    elif family == 'output':
        identity, topology, w, h, num, den, x, y = values
        require(identity > 0 and topology == 1, 'output identity/topology')
        require((w, h) in ((1280, 720), (1920, 1080)), 'output mode')
        require((num, den) in ((1, 1), (3, 2), (2, 1)) and x == y == 0, 'output transform')


def encode(family, values, *, sequence=1, request=2, resource=3, output=4, object_id=5, label=b''):
    require(family in BODY and type(label) is bytes, 'endpoint/label type')
    require(all(type(v) is int for v in (*values, sequence, request, resource, output, object_id)), 'integer type')
    require(min(sequence, request, resource, output, object_id) > 0, 'zero header identity')
    body_check(family, values, label)
    require(family == 'semantic' or not label, 'unexpected tail')
    try:
        body = BODY[family].pack(*values) + label
        return HEADER.pack(b'CDU0', 0, 0, 64 + len(body), 1, 0, sequence, request, resource, output, object_id, len(body), 0) + body
    except (struct.error, OverflowError) as error:
        raise ValueError('field width') from error


def decode(family, data, *, sequence=1, resource=3, output=4):
    require(family in BODY and type(data) is bytes and 64 <= len(data) <= 4096, 'envelope bounds')
    magic, major, minor, total, kind, flags, seq, request, generation, out, object_id, size, reserved = HEADER.unpack_from(data)
    require((magic, major, minor, kind, flags, reserved) == (b'CDU0', 0, 0, 1, 0, 0), 'unsupported envelope')
    require(total == len(data) == 64 + size, 'envelope length')
    require(seq == sequence > 0 and generation == resource > 0 and out == output > 0 and request > 0 and object_id > 0, 'stale/zero context')
    schema = BODY[family]
    require(size >= schema.size, 'truncated body')
    values = schema.unpack_from(data, 64)
    label = data[64 + schema.size:]
    require(family == 'semantic' or not label, 'trailing bytes')
    body_check(family, values, label)
    return values, label


def encode_dev(payload, *, sequence=1, kind=1):
    require(type(payload) is bytes and len(payload) <= 1048576 and type(sequence) is int and 0 < sequence < 2**64 and type(kind) is int and 1 <= kind <= 12, 'dev bounds')
    return DEV.pack(b'OSU1', 1, 0, 32, len(payload), sequence, kind, 0, 0) + payload


def decode_dev(data, *, sequence=1, development=False, authenticated=False):
    require(type(data) is bytes and 32 <= len(data) <= 1048608, 'dev frame bound')
    magic, major, minor, header, size, seq, kind, flags, reserved = DEV.unpack_from(data)
    require((magic, major, minor, header, flags, reserved) == (b'OSU1', 1, 0, 32, 0, 0), 'dev version/flags')
    require(size <= 1048576 and len(data) == 32 + size and seq == sequence > 0 and 1 <= kind <= 12, 'dev length/sequence/kind')
    require(development is True and (kind <= 2 or authenticated is True), 'disabled/unauthenticated dev mutation')
    # This Boolean models a trusted session-layer result for draft tests only;
    # it is never a wire credential or a supplied cryptographic construction.
    return kind, data[32:]
