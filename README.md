# fixPath

`fixPath` is a tool to modify the search locations for DLLs (Dynamic Shared Objects) for [Microsoft Windows 
Executables](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format) by rewriting parts of the binary's PE 
header when the `.fixPath` section is present and indicates support for such rewrite.

> fix - fasten (something) securely in a particular place or position.

List all imports:

    $ fixPath.exe --list-imports
     - (foo.dll)
     - (bar.dll) -> c:\foo\bar.dll

Change the `foo.dll` import location to an absolute path:

    $ fixPath.exe --set-import foo.dll c:\test\foo.dll

Use `fixPath` to modify the library search path:
* **[Microsoft's linker default search path(s)](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order).**, i.e. c:\Windows\System32 and similar
* **relative path** to the executable: lib\foo.dll
* **absoluate path** in filesystem: c:\lib\foo.dll

Note: `fixPath` preserves the original library name and appends this information to the `.idata` section of the PE header.

## fixPath requirements

`fixPath` requires special support from the linker `lld` - **the LLVM linker**. It can't be applied to already existing binaries.

## How does `fixPath` work?

The `fixPath` tool relies on a custom PE extension section called `.fixPath` (a section like `.idata` or `.didata`).

A patched version of the **ldd linker** or **ld linker** will create this section: 
* when passing `-fixPath` on the command line the `.fixPath` section is created
* when passing the `-dllname_max_size=500` on the command line, the `dllname_max_size` can be set

The `fixPath` tool will check if the `.fixPath` field exists and if so one can use it to change the loader order from:

* The **library** filename, for instance `KERNEL32.dll` will be searched in the default locations, like `C:\Windows\System32`.
* However, it can be a **relative filepath** like `..\foo.dll` or
* It can be an **absolute filepath** like `c:\bar.dll`

### .fixPath v1

This `.fixPath` section contains:
* a `version field`, `dllname_max_size` [u32], 8 bytes padding 
* `idataNameTable_size` [u32]
* array [`idataNameTable dllname`] [ascii]
* 4 bytes padding
* `didataNameTable_size` [u32]
* array [`didataNameTable dllname`] [ascii],
* 4 bytes padding
* URL of fixPath https://github.com/nixcloud/fixPath

## LLM (LLVM compiler) support

See https://github.com/qknight/llvm-project/tree/libnix_PE-fixPath

Features:

* [x] Reserving 300 chars dllname space
* [x] Adding `.fixPath` section with **version** and **dllname size**
* [ ] Adding `.fixPath` section with **dllnames** for dll loading
* [ ] Adding `.fixPath` section with **dllnames** for delayed dll loading 
* [x] Using llm options to represent defaults
* [ ] Command line switches for cmake & similar
* [ ] Command line switches for clang-cl
* [ ] llm library switches

## Other linkers 

* [ ] GNU LD (GNU compiler) support
* [ ] GNU gold (GNU compiler) support
* [ ] [mold](https://github.com/rui314/mold) support
* [ ] Visual Studio linker support

No prototypes yet.

# Tests

The `tests/` directory contains a set of executables which were used for verifying the fixPath execution.

## Research - manual editing (010 editor / ImHex)

Modifying main.exe's library entry (without extending the .rdata)

* [x] works for absolute path in main.exe (self built program using a dll)
* [x] works for relative path in main.exe (self built program using a dll)

* [x] works with long filename : "c:\t\aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.dll"
* [x] works with long directory: "c:\aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\t.dll" 
* [x] works with "C:\t\nix\store\zxxialnsgv0ahms5d35sivqzxqg1kicf-libiec61883-1.2.0\lib\aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.dll" 213 chars
* [x] works with "C:\t\nix\store\zxxialnsgv0ahms5d35sivqzxqg1kicf-libiec61883-1.2.0\lib\aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.dll" 269 chars 

# FAQ

## What motivates `fixPath`?

When compiling software using the [nix package manager](https://nixos.org), it is required to build the software from a
different folder than the final location of the software. Using `fixPath`, which is similar to the **rpath** feature
from Linux/Unix the developer can hard-code the paths where the libraries are loaded from.

It basically prevents DLL-Hell by making it possible to use absolute paths for certain libraries and not rely on
the [Microsoft's linker default search path(s)](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order).

## Why not modify any EXE file?

Modifying _dll paths of arbitrary executables_ proved way too complicated and often times resulted in corrupt
programs, because the required relocations of sections and addresses inside the sections is undocumented
and not wanted.

## rpath on Windows discussions

See:

* https://developercommunity.visualstudio.com/idea/566616/support-rpath-for-binaries-during-development.html
* https://stackoverflow.com/questions/107888/is-there-a-windows-msvc-equivalent-to-the-rpath-linker-flag
