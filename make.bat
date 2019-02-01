@echo off

mkdir bin > NUL 2>&1
del bin\*.exe > NUL 2>&1

set WINKIT=C:\Program Files (x86)\Windows Kits\10
set WINVER=10.0.17134.0

set WINBIN=%WINKIT%\bin\%WINVER%\x64
set WINLIB=%WINKIT%\Lib\%WINVER%\um\x64
set WININC=%WINKIT%\Include\%WINVER%\um
set COMMON=cpp\utils.cpp cpp\point.cpp cpp\color.cpp cpp\input.cpp

clang -Xclang -flto-visibility-public-std ^
  -I"%WININC%" -l"%WINLIB%\User32.lib" -l"%WINLIB%\WS2_32.lib" -l"%WINLIB%\Gdi32.lib" ^
  -Iinclude %COMMON% cpp\action.cpp cpp\poe.cpp -obin\poe.exe ^
  > build.log 2>&1

clang -Xclang -flto-visibility-public-std ^
  -I"%WININC%" -l"%WINLIB%\User32.lib" -l"%WINLIB%\WS2_32.lib" -l"%WINLIB%\Gdi32.lib" ^
  -Iinclude %COMMON% cpp\memu-helper.cpp -obin\memu-helper.exe ^
  >> build.log 2>&1

"%WINBIN%\mt.exe" -manifest poe.exe.manifest -outputresource:bin\poe.exe;#1 >> build.log 2>&1
