; Winsentials NSIS Installer Script
; Modern UI 2 with High-DPI, Solid LZMA Compression, and 64-bit Architecture

!ifndef VERSION
  !define VERSION "0.1.0"
!endif

!ifndef OUTFILE
  !define OUTFILE "..\..\target\release\winsentials-win-x64-setup.exe"
!endif

!ifndef BINARY_PATH
  !define BINARY_PATH "..\..\target\release\Winsentials.exe"
!endif

Unicode true
SetCompressor /SOLID lzma
RequestExecutionLevel admin

Name "Winsentials"
OutFile "${OUTFILE}"
InstallDir "$PROGRAMFILES64\Winsentials"
InstallDirRegKey HKLM "Software\Winsentials" "InstallDir"

; Modern UI 2 Configuration
!include "MUI2.nsh"
!include "FileFunc.nsh"

!define MUI_ICON "..\..\assets\app-logo.ico"
!define MUI_UNICON "..\..\assets\app-logo.ico"

!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_RIGHT
!define MUI_HEADERIMAGE_BITMAP "..\..\assets\app-installer-header.bmp"
!define MUI_WELCOMEFINISHPAGE_BITMAP "..\..\assets\app-installer-sidebar.bmp"
!define MUI_ABORTWARNING

; Dark Theme Styling (matching Winsentials Arclate Dark #0F151A / #E9EEF1)
!define MUI_BGCOLOR "0F151A"
!define MUI_TEXTCOLOR "E9EEF1"

; Installer Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

!define MUI_FINISHPAGE_RUN "$INSTDIR\Winsentials.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Запустить Winsentials"
!insertmacro MUI_PAGE_FINISH

; Uninstaller Pages
!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; Languages
!insertmacro MUI_LANGUAGE "Russian"
!insertmacro MUI_LANGUAGE "English"

Section "Winsentials" SecMain
    SectionIn RO
    SetOutPath "$INSTDIR"

    ; Close running instances if any
    DetailPrint "Проверка и завершение запущенных процессов Winsentials..."
    nsExec::Exec 'taskkill /F /IM Winsentials.exe /T'

    File "/oname=Winsentials.exe" "${BINARY_PATH}"

    ; Write Uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    ; Create Shortcuts
    CreateDirectory "$SMPROGRAMS\Winsentials"
    CreateShortcut "$SMPROGRAMS\Winsentials\Winsentials.lnk" "$INSTDIR\Winsentials.exe" "" "$INSTDIR\Winsentials.exe" 0
    CreateShortcut "$SMPROGRAMS\Winsentials\Удалить Winsentials.lnk" "$INSTDIR\Uninstall.exe" "" "$INSTDIR\Uninstall.exe" 0
    CreateShortcut "$DESKTOP\Winsentials.lnk" "$INSTDIR\Winsentials.exe" "" "$INSTDIR\Winsentials.exe" 0

    ; Write Add/Remove Programs Registry Entries
    WriteRegStr HKLM "Software\Winsentials" "InstallDir" "$INSTDIR"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Winsentials" "DisplayName" "Winsentials"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Winsentials" "DisplayVersion" "${VERSION}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Winsentials" "Publisher" "Noktomezo"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Winsentials" "URLInfoAbout" "https://github.com/Noktomezo/Winsentials"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Winsentials" "DisplayIcon" "$INSTDIR\Winsentials.exe,0"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Winsentials" "UninstallString" '"$INSTDIR\Uninstall.exe"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Winsentials" "QuietUninstallString" '"$INSTDIR\Uninstall.exe" /S'
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Winsentials" "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Winsentials" "NoRepair" 1

    ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
    IntFmt $0 "0x%08X" $0
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Winsentials" "EstimatedSize" "$0"
SectionEnd

Section "Uninstall"
    DetailPrint "Завершение процессов Winsentials..."
    nsExec::Exec 'taskkill /F /IM Winsentials.exe /T'

    Delete "$DESKTOP\Winsentials.lnk"
    Delete "$SMPROGRAMS\Winsentials\Winsentials.lnk"
    Delete "$SMPROGRAMS\Winsentials\Удалить Winsentials.lnk"
    RMDir "$SMPROGRAMS\Winsentials"

    Delete "$INSTDIR\Winsentials.exe"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"

    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Winsentials"
    DeleteRegKey HKLM "Software\Winsentials"
SectionEnd
