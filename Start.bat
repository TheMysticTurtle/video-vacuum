@echo off
rem ===========================================================================
rem  Video Saver - Windows launcher. Just double-click this file.
rem
rem  1. Checks that Python, yt-dlp, ffmpeg and deno are installed.
rem  2. Offers to install anything missing with winget (built into Windows).
rem  3. Starts the app and opens it in your browser.
rem ===========================================================================
setlocal EnableExtensions
cd /d "%~dp0"
title Video Saver

rem --- Python 3.9 or newer? (the Microsoft Store "python" placeholder fails this test)
set "PY="
python -c "import sys; sys.exit(sys.version_info < (3, 9))" >nul 2>&1 && set "PY=python"
if not defined PY py -c "import sys; sys.exit(sys.version_info < (3, 9))" >nul 2>&1 && set "PY=py"

rem --- Which tools are missing? Each is installed with its own winget package.
set "MISSING="
set "PACKAGES="
if not defined PY (
    set "MISSING=%MISSING% Python"
    set "PACKAGES=%PACKAGES% 9NQ7512CXL7T"
)
where yt-dlp >nul 2>&1
if errorlevel 1 (
    rem The yt-dlp package also brings in ffmpeg and deno.
    set "MISSING=%MISSING% yt-dlp"
    set "PACKAGES=%PACKAGES% yt-dlp.yt-dlp"
    goto :check_done
)
where ffmpeg >nul 2>&1
if errorlevel 1 (
    set "MISSING=%MISSING% ffmpeg"
    set "PACKAGES=%PACKAGES% yt-dlp.FFmpeg"
)
where deno >nul 2>&1
if errorlevel 1 (
    set "MISSING=%MISSING% deno"
    set "PACKAGES=%PACKAGES% DenoLand.Deno"
)
:check_done

if not defined MISSING goto :run

rem --- Something is missing: offer to install it.
echo.
echo  Video Saver needs:%MISSING%
echo.
where winget >nul 2>&1
if errorlevel 1 goto :no_winget

choice /c YN /m " Install it now with winget"
if errorlevel 2 goto :no_install

for %%P in (%PACKAGES%) do winget install --id %%P --exact
echo.
echo  Done. Close this window and double-click Start.bat again
echo  so Windows picks up the newly installed programs.
echo.
pause
exit /b 0

:no_winget
echo  winget isn't available on this PC. Install these by hand:
echo    Python:  https://www.python.org/downloads/
echo    yt-dlp:  https://github.com/yt-dlp/yt-dlp#installation
echo  Then double-click Start.bat again.
echo.
pause
exit /b 1

:no_install
echo  Nothing was installed. See README.md for manual install steps.
echo.
pause
exit /b 1

rem --- Everything is ready: start the app. This window stays open while it runs.
:run
%PY% app.py
if errorlevel 1 pause
