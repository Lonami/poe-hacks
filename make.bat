@echo off
del *.exe > NUL 2>&1

set WINKIT=C:\Program Files (x86)\Windows Kits\10
set WINVER=10.0.17134.0

set WINBIN=%WINKIT%\bin\%WINVER%\x64
set WINLIB=%WINKIT%\Lib\%WINVER%\um\x64
set WININC=%WINKIT%\Include\%WINVER%\um

clang -Xclang -flto-visibility-public-std ^
  -I"%WININC%" -l"%WINLIB%\User32.lib" -l"%WINLIB%\WS2_32.lib" -l"%WINLIB%\Gdi32.lib" ^
  utils.cpp input.cpp poe.cpp -opoe.exe ^
  > build.log 2>&1

clang -Xclang -flto-visibility-public-std ^
  -I"%WININC%" -l"%WINLIB%\User32.lib" -l"%WINLIB%\WS2_32.lib" -l"%WINLIB%\Gdi32.lib" ^
  utils.cpp input.cpp memu-helper.cpp -omemu-helper.exe ^
  >> build.log 2>&1

"%WINBIN%\mt.exe" -manifest poe.exe.manifest -outputresource:poe.exe;#1 >> build.log 2>&1
