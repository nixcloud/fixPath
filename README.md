# fixPath

`fixPath` is a tool to modify the search locations for DLLs (Dynamic Shared Objects) for [Microsoft Windows 
Executables](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format) by rewriting parts of the binary's PE 
header where the `.fixPath` section is present and indicates support for such rewrite.

List all imports:

    $ fixPath.exe --list-imports
     - (foo.dll)
     - (bar.dll) -> c:\foo\bar.dll

Change the `foo.dll` import location to an absolute path:

    $ fixPath.exe --set-import foo.dll c:\test\foo.dll

Use `fixPath` to modify the library search path:
* **default linker search path**, i.e. c:\Windows\System32 and similar
* **relative path** to the executable: lib\foo.dll
* **absoluate path** in filesystem: c:\lib\foo.dll

Note: `fixPath` preserves the original library name and appends this information to the `.idata` section of the PE header.

## Internals

How does this work? The `fixDLLPath` tool relies on a custom PE extension section called `.fixPath` similar to `.idata` or `.didata`.

A patched version of the **ldd linker** or **ld linker** will create this section: 
* when passing `-fixPath` on the command line the `.fixPath` section is created
* when passing the `-dllname_max_size=500` on the command line, the `dllname_max_size` can be set

This `.fixPath` section contains:
* a `version field`, `dllname_max_size` [u32], 8 bytes padding 
* `idataNameTable_size` [u32]
* array [`idataNameTable dllname`] [ascii]
* 4 bytes padding
* `didataNameTable_size` [u32]
* array [`didataNameTable dllname`] [ascii],
* 4 bytes padding
* URL of fixPath https://github.com/qknight/fixPath

The `fixDLLPath` tool will check if the `.fixPath` field exists and if so one can use it to change the loader order from:

* The **library** filename, for instance `KERNEL32.dll` will be searched in the default locations, like `C:\Windows\System32`. 
* However, it can be a **relative filepath** like `..\foo.dll` or 
* It can be an **absolute filepath** like `c:\bar.dll`




# coverage

if the space, for the new full path dll in rdata, is not enough: remove the dll name with 00 and append it to the end of rdata field

## fixPath

* [ ] works for absolute path in calc.exe
* [ ] works for relative path in calc.exe
* [ ] works for absolute path in main.exe (self built program using a dll)
* [ ] works for relative path in main.exe (self built program using a dll)

https://www.hexacorn.com/blog/2016/12/15/pe-section-names-re-visited/

## manual editing (010 editor)

modifying main.exe's library entry (without extending the .rdata)

* [x] works for absolute path in main.exe (self built program using a dll)
* [x] works for relative path in main.exe (self built program using a dll)

* [x] works with long filename : "c:\t\aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.dll"
* [x] works with long directory: "c:\aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\t.dll" 
* [x] works with "C:\t\nix\store\zxxialnsgv0ahms5d35sivqzxqg1kicf-libiec61883-1.2.0\lib\aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.dll" 213 chars
* [x] works with "C:\t\nix\store\zxxialnsgv0ahms5d35sivqzxqg1kicf-libiec61883-1.2.0\lib\aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.dll" 269 chars 




# tests

This directory contains a set of executables which were used for verifying the fixPath execution