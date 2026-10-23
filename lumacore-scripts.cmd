@echo off
:: Root entrypoint for the script launcher TUI (Windows).
:: On Unix, use ./lumacore-scripts instead.

setlocal
pushd "%~dp0"

if "%~1"=="--help" goto :help
if "%~1"=="-h" goto :help
if "%~1"=="--release" goto :release

set PROFILE=debug
set BIN=.\target\debug\lumacore-scripts.exe
set BUILDARGS=
goto :build

:release
set PROFILE=release
set BIN=.\target\release\lumacore-scripts.exe
set BUILDARGS=--release
shift

:build
if exist "%BIN%" goto :run
echo Building lumacore-scripts (%PROFILE%)...
cargo build %BUILDARGS% -p lumacore-scripts
if not exist "%BIN%" (
    echo Failed to build %BIN%
    popd
    exit /b 1
)

:run
"%BIN%" %*
popd
exit /b %errorlevel%

:help
echo LumaCore scripts launcher
echo.
echo Usage:
echo   lumacore-scripts
echo   lumacore-scripts --release
echo.
echo This launches the ratatui menu for the canonical scripts.
echo   Unix scripts:   scripts/unix/
echo   Windows scripts: scripts/win/
echo The launcher builds the matching workspace binary in target/ if needed.
popd
exit /b 0
