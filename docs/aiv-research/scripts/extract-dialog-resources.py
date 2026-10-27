#!/usr/bin/env python3
"""List Win32 dialog resources from a PE using the Windows resource API."""

from __future__ import annotations

import argparse
import ctypes
import json
import struct
from ctypes import wintypes


LOAD_LIBRARY_AS_DATAFILE = 0x00000002
LOAD_LIBRARY_AS_IMAGE_RESOURCE = 0x00000020
RT_DIALOG = 5
DS_SETFONT = 0x00000040


def align(value: int, boundary: int = 4) -> int:
    return (value + boundary - 1) & ~(boundary - 1)


def u16(data: bytes, off: int) -> tuple[int, int]:
    return struct.unpack_from("<H", data, off)[0], off + 2


def s16(data: bytes, off: int) -> tuple[int, int]:
    return struct.unpack_from("<h", data, off)[0], off + 2


def u32(data: bytes, off: int) -> tuple[int, int]:
    return struct.unpack_from("<I", data, off)[0], off + 4


def variable(data: bytes, off: int) -> tuple[object | None, int]:
    first, off = u16(data, off)
    if first == 0:
        return None, off
    if first == 0xFFFF:
        ordinal, off = u16(data, off)
        return {"ordinal": ordinal}, off
    chars = [first]
    while True:
        char, off = u16(data, off)
        if char == 0:
            break
        chars.append(char)
    return "".join(chr(char) for char in chars), off


def parse_dialog(data: bytes) -> dict:
    extended = len(data) >= 4 and struct.unpack_from("<HH", data, 0) == (1, 0xFFFF)
    off = 0
    result: dict[str, object] = {"extended": extended, "controls": []}

    if extended:
        _, off = u16(data, off)
        _, off = u16(data, off)
        help_id, off = u32(data, off)
        ex_style, off = u32(data, off)
        style, off = u32(data, off)
        count, off = u16(data, off)
        result.update(help_id=help_id, ex_style=ex_style, style=style)
    else:
        style, off = u32(data, off)
        ex_style, off = u32(data, off)
        count, off = u16(data, off)
        result.update(ex_style=ex_style, style=style)

    x, off = s16(data, off)
    y, off = s16(data, off)
    cx, off = s16(data, off)
    cy, off = s16(data, off)
    result.update(x=x, y=y, cx=cx, cy=cy, control_count=count)
    result["menu"], off = variable(data, off)
    result["class"], off = variable(data, off)
    result["title"], off = variable(data, off)

    if style & DS_SETFONT:
        point_size, off = u16(data, off)
        font: dict[str, object] = {"point_size": point_size}
        if extended:
            weight, off = u16(data, off)
            italic = data[off]
            charset = data[off + 1]
            off += 2
            font.update(weight=weight, italic=italic, charset=charset)
        font["face"], off = variable(data, off)
        result["font"] = font

    controls = result["controls"]
    assert isinstance(controls, list)
    for _ in range(count):
        off = align(off)
        control: dict[str, object] = {}
        if extended:
            control["help_id"], off = u32(data, off)
            control["ex_style"], off = u32(data, off)
            control["style"], off = u32(data, off)
        else:
            control["style"], off = u32(data, off)
            control["ex_style"], off = u32(data, off)
        control["x"], off = s16(data, off)
        control["y"], off = s16(data, off)
        control["cx"], off = s16(data, off)
        control["cy"], off = s16(data, off)
        if extended:
            control["id"], off = u32(data, off)
        else:
            control["id"], off = u16(data, off)
        control["class"], off = variable(data, off)
        control["title"], off = variable(data, off)
        extra, off = u16(data, off)
        control["extra_bytes"] = extra
        off += extra
        controls.append(control)
    return result


def enum_dialogs(path: str) -> list[dict]:
    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel32.LoadLibraryExW.argtypes = [wintypes.LPCWSTR, wintypes.HANDLE, wintypes.DWORD]
    kernel32.LoadLibraryExW.restype = wintypes.HMODULE
    kernel32.EnumResourceNamesW.argtypes = [wintypes.HMODULE, ctypes.c_void_p, ctypes.c_void_p, wintypes.LPARAM]
    kernel32.EnumResourceNamesW.restype = wintypes.BOOL
    kernel32.FindResourceW.argtypes = [wintypes.HMODULE, ctypes.c_void_p, ctypes.c_void_p]
    kernel32.FindResourceW.restype = wintypes.HRSRC
    kernel32.SizeofResource.argtypes = [wintypes.HMODULE, wintypes.HRSRC]
    kernel32.SizeofResource.restype = wintypes.DWORD
    kernel32.LoadResource.argtypes = [wintypes.HMODULE, wintypes.HRSRC]
    kernel32.LoadResource.restype = wintypes.HGLOBAL
    kernel32.LockResource.argtypes = [wintypes.HGLOBAL]
    kernel32.LockResource.restype = wintypes.LPVOID
    kernel32.FreeLibrary.argtypes = [wintypes.HMODULE]

    module = kernel32.LoadLibraryExW(path, None, LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE)
    if not module:
        raise ctypes.WinError(ctypes.get_last_error())

    names: list[int | str] = []
    callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HMODULE, ctypes.c_void_p, ctypes.c_void_p, wintypes.LPARAM)

    def callback(_module, _kind, name, _param):
        raw = int(name or 0)
        names.append(raw if raw <= 0xFFFF else ctypes.wstring_at(raw))
        return True

    callback_ref = callback_type(callback)
    try:
        if not kernel32.EnumResourceNamesW(module, ctypes.c_void_p(RT_DIALOG), callback_ref, 0):
            error = ctypes.get_last_error()
            if error:
                raise ctypes.WinError(error)
        dialogs = []
        for name in names:
            name_ptr = ctypes.c_void_p(name) if isinstance(name, int) else ctypes.cast(ctypes.c_wchar_p(name), ctypes.c_void_p)
            resource = kernel32.FindResourceW(module, name_ptr, ctypes.c_void_p(RT_DIALOG))
            size = kernel32.SizeofResource(module, resource)
            loaded = kernel32.LoadResource(module, resource)
            pointer = kernel32.LockResource(loaded)
            parsed = parse_dialog(ctypes.string_at(pointer, size))
            parsed["resource_id"] = name
            parsed["resource_size"] = size
            dialogs.append(parsed)
        return dialogs
    finally:
        kernel32.FreeLibrary(module)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("pe")
    parser.add_argument("--contains", help="Only dialogs containing this case-insensitive text")
    args = parser.parse_args()
    dialogs = enum_dialogs(args.pe)
    if args.contains:
        needle = args.contains.casefold()
        dialogs = [dialog for dialog in dialogs if needle in json.dumps(dialog, ensure_ascii=False).casefold()]
    print(json.dumps(dialogs, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
