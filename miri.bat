@echo off
setlocal
REM set MIRIFLAGS=-Zmiri-tree-borrows
cargo +nightly miri test
endlocal