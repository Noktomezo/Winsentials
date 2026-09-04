Option Explicit

If WScript.Arguments.Count < 1 Then
    WScript.Quit 1
End If

Dim scriptPath, filePath, command
scriptPath = Left(WScript.ScriptFullName, Len(WScript.ScriptFullName) - 4) & ".ps1"
filePath = WScript.Arguments(0)
command = "powershell.exe -NoLogo -NoProfile -NonInteractive -STA -ExecutionPolicy Bypass -File """ _
    & scriptPath & """ -ImagePath """ & filePath & """"

CreateObject("WScript.Shell").Run command, 0, False
