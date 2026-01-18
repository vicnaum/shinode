# libmdbx

## Purpose
Vendored libmdbx C/C++ source and build files.

## Contents (one hop)
### Subdirectories
- [x] `cmake/` - CMake helper modules.
- [x] `man1/` - Command-line tool man pages.

### Files
- `mdbx.c`, `mdbx.c++` - Core libmdbx implementation.
- `mdbx.h`, `mdbx.h++` - Public C/C++ headers.
- `CMakeLists.txt`, `Makefile`, `GNUmakefile` - Build scripts.
- `config.h.in` - Build configuration template.
- `VERSION.json` - Upstream version metadata.
- `LICENSE`, `NOTICE` - Licensing and notices.
- `mdbx_chk.c`, `mdbx_copy.c`, `mdbx_drop.c`, `mdbx_dump.c`, `mdbx_load.c`, `mdbx_stat.c` - CLI tools.
- `ntdll.def` - Windows export definitions.

## Relationships
- **Used by**: `mdbx-sys` build to compile the native library.
