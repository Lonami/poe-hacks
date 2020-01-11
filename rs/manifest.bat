@echo off

set WINKIT=C:\Program Files (x86)\Windows Kits\10
set WINVER=10.0.18362.0
set WINBIN=%WINKIT%\bin\%WINVER%\x64

copy target\release\poe.exe poe.exe
"%WINBIN%\mt.exe" -manifest ..\poe.exe.manifest -outputresource:poe.exe;#1
