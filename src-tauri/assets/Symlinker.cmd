@echo off
set "myCommand=(new-object -COM 'Shell.Application').BrowseForFolder(0,'',0).self.path"
for /f "usebackq delims=" %%I in (`powershell -NoProfile -NonInteractive -Command "%myCommand%"`) do set "dir=%%I"
if not defined dir exit /b 1
setlocal enabledelayedexpansion
mklink "!dir!\%~nx1" "%~1"
