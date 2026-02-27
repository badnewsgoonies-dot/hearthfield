#!/usr/bin/env python3
"""Strip the 'target_features' custom section from a WASM binary.

This forces wasm-bindgen to fall back to slab-based object passing
instead of externref, improving mobile browser compatibility.
"""
import sys
import struct


def read_leb128(data, offset):
    result = 0
    shift = 0
    i = offset
    while True:
        byte = data[i]
        result |= (byte & 0x7F) << shift
        i += 1
        if byte & 0x80 == 0:
            break
        shift += 7
    return result, i - offset


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <input.wasm> <output.wasm>", file=sys.stderr)
        sys.exit(1)

    data = open(sys.argv[1], "rb").read()
    assert data[:4] == b"\x00asm", "Not a WASM binary"

    out = bytearray(data[:8])
    pos = 8
    stripped = False

    while pos < len(data):
        section_start = pos
        section_id = data[pos]
        pos += 1
        section_size, leb_len = read_leb128(data, pos)
        pos += leb_len
        section_end = pos + section_size

        if section_id == 0:
            name_len, name_leb_len = read_leb128(data, pos)
            name_start = pos + name_leb_len
            name = data[name_start : name_start + name_len].decode("utf-8", errors="replace")
            if name == "target_features":
                print(f"  Stripped 'target_features' ({section_end - section_start} bytes)", file=sys.stderr)
                stripped = True
                pos = section_end
                continue

        out.extend(data[section_start:section_end])
        pos = section_end

    open(sys.argv[2], "wb").write(out)
    if not stripped:
        print("  Warning: no target_features section found", file=sys.stderr)


if __name__ == "__main__":
    main()
