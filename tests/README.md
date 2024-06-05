# program_with_lib

This repository contains a `C project with a main function and a shared library (dll)` 
which is compiled into an `exe` and a `dll` for testing `fixPath`, a util to change the `dllName` field
inside the `.idata/.rdata` of the `PE` header. 

# build custom `lld`

to compile LLD from https://github.com/qknight/llvm-project/tree/libnix_PE-fixPath

* windows 10
* powershell 7
* cmake version 3.24.1
* LLVM-18.1.6-win64.exe
* Microsoft Visual Studio Community 2022 (64-bit) - Current Version 17.9.7
* ImHex 1.33.2

https://discourse.llvm.org/t/build-lld-on-windows-10-using-ninja-llvm-18-1-6-cmake/79318/5

## build test with custom `lld`

The build system shows how LLD can be set to support fixPath:

    cd tests/program_with_lib/  
    mkdir build; cd build
    cmake -G "Ninja" -D_CMAKE_TOOLCHAIN_PREFIX=llvm- ..
    ninja --verbose 
    ./test_mylib.exe 
    objdump.exe -h ./tes_mylib.exe
    objdump.exe -h ./mylib.dll

It is a test for the `lld` linker, as it should add the `.fixPath` section.

## test_mylib.exe

```
objdump.exe -h .\test_mylib.exe
.\test_mylib.exe:     file format pei-x86-64

Sections:
Idx Name          Size      VMA               LMA               File off  Algn
0 .text         00002366  0000000140001000  0000000140001000  00000400  2**4
CONTENTS, ALLOC, LOAD, READONLY, CODE
1 .rdata        00001334  0000000140004000  0000000140004000  00002800  2**4
CONTENTS, ALLOC, LOAD, READONLY, DATA
2 .data         00000200  0000000140006000  0000000140006000  00003c00  2**4
CONTENTS, ALLOC, LOAD, DATA
3 .pdata        0000039c  0000000140007000  0000000140007000  00003e00  2**2
CONTENTS, ALLOC, LOAD, READONLY, DATA
4 .fixPath      0000007b  0000000140008000  0000000140008000  00004200  2**2
CONTENTS, ALLOC, LOAD, DATA
5 .rsrc         000001a8  0000000140009000  0000000140009000  00004400  2**2
CONTENTS, ALLOC, LOAD, READONLY, DATA
6 .reloc        00000030  000000014000a000  000000014000a000  00004600  2**2
CONTENTS, ALLOC, LOAD, READONLY, DATA
```

## mylib.dll
```
objdump.exe -h .\mylib.dll

.\mylib.dll:     file format pei-x86-64

Sections:
Idx Name          Size      VMA               LMA               File off  Algn
0 .text         000026d6  0000000180001000  0000000180001000  00000400  2**4
CONTENTS, ALLOC, LOAD, READONLY, CODE
1 .rdata        00001114  0000000180004000  0000000180004000  00002c00  2**4
CONTENTS, ALLOC, LOAD, READONLY, DATA
2 .data         00000200  0000000180006000  0000000180006000  00003e00  2**4
CONTENTS, ALLOC, LOAD, DATA
3 .pdata        000003fc  0000000180007000  0000000180007000  00004000  2**2
CONTENTS, ALLOC, LOAD, READONLY, DATA
4 .fixPath      00000071  0000000180008000  0000000180008000  00004400  2**2
CONTENTS, ALLOC, LOAD, DATA
5 .rsrc         000001a8  0000000180009000  0000000180009000  00004600  2**2
CONTENTS, ALLOC, LOAD, READONLY, DATA
6 .reloc        00000028  000000018000a000  000000018000a000  00004800  2**2
CONTENTS, ALLOC, LOAD, READONLY, DATA
```

