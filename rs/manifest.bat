@echo off

set WINKIT=C:\Program Files (x86)\Windows Kits\10
set WINVER=10.0.18362.0
set WINBIN=%WINKIT%\bin\%WINVER%\x64

"%WINBIN%\mt.exe" -manifest ..\poe.exe.manifest -outputresource:target\release\poe.exe;#1
