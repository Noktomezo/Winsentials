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
!include "LogicLib.nsh"

!define MUI_ICON "..\..\assets\app-logo.ico"
!define MUI_UNICON "..\..\assets\app-logo.ico"

!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_RIGHT
!define MUI_HEADERIMAGE_BITMAP "..\..\assets\app-installer-header.bmp"
!define MUI_WELCOMEFINISHPAGE_BITMAP "..\..\assets\app-installer-sidebar.bmp"
!define MUI_UNWELCOMEFINISHPAGE_BITMAP "..\..\assets\app-installer-sidebar.bmp"
!define MUI_ABORTWARNING

; Dark Theme Styling (matching Winsentials Arclate Dark #0F151A / #E9EEF1)
!define MUI_BGCOLOR "0F151A"
!define MUI_TEXTCOLOR "E9EEF1"

!define MUI_CUSTOMFUNCTION_GUIINIT onGUIInit
!define MUI_CUSTOMFUNCTION_UNGUIINIT un.customUnGUIInit

; Installer Pages
!define MUI_PAGE_CUSTOMFUNCTION_SHOW onPageShow
!insertmacro MUI_PAGE_WELCOME

!define MUI_PAGE_CUSTOMFUNCTION_SHOW onPageShow
!insertmacro MUI_PAGE_DIRECTORY

!define MUI_PAGE_CUSTOMFUNCTION_SHOW onPageShow
!insertmacro MUI_PAGE_INSTFILES

!define MUI_PAGE_CUSTOMFUNCTION_SHOW onPageShow
!define MUI_FINISHPAGE_RUN "$INSTDIR\Winsentials.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Запустить Winsentials"
!insertmacro MUI_PAGE_FINISH

; Uninstaller Pages
!define MUI_PAGE_CUSTOMFUNCTION_SHOW un.unPageShow
!insertmacro MUI_UNPAGE_WELCOME

!define MUI_PAGE_CUSTOMFUNCTION_SHOW un.unPageShow
!insertmacro MUI_UNPAGE_CONFIRM

!define MUI_PAGE_CUSTOMFUNCTION_SHOW un.unPageShow
!insertmacro MUI_UNPAGE_INSTFILES

!define MUI_PAGE_CUSTOMFUNCTION_SHOW un.unPageShow
!insertmacro MUI_UNPAGE_FINISH

; Languages
!insertmacro MUI_LANGUAGE "Russian"
!insertmacro MUI_LANGUAGE "English"

; Dark Theme Painter Functions
Function onGUIInit
    ; 1. Dark titlebar (DWMWA_USE_IMMERSIVE_DARK_MODE = 20)
    System::Call 'dwmapi::DwmSetWindowAttribute(p $HWNDPARENT, i 20, *i 1, i 4)'
    ; 2. Parent dialog background (bottom button bar)
    SetCtlColors $HWNDPARENT 0xE9EEF1 0x0F151A
FunctionEnd

Function onPageShow
    ; 1. Dark titlebar
    System::Call 'dwmapi::DwmSetWindowAttribute(p $HWNDPARENT, i 20, *i 1, i 4)'
    ; 2. Outer parent dialog (buttons area)
    SetCtlColors $HWNDPARENT 0xE9EEF1 0x0F151A
    
    ; 3. Branding text
    GetDlgItem $1 $HWNDPARENT 1028
    ${If} $1 != 0
        SetCtlColors $1 0x7E8C9A 0x0F151A
    ${EndIf}

    ; 4. Inner dialog & all child controls
    FindWindow $0 "#32770" "" $HWNDPARENT
    ${If} $0 != 0
        SetCtlColors $0 0xE9EEF1 0x0F151A
        
        StrCpy $2 1000
        ${While} $2 <= 1040
            GetDlgItem $1 $0 $2
            ${If} $1 != 0
                ${If} $2 == 1019
                    ; Edit box for path: bg #1A2228, text #E9EEF1
                    SetCtlColors $1 0xE9EEF1 0x1A2228
                ${Else}
                    SetCtlColors $1 0xE9EEF1 0x0F151A
                ${EndIf}
            ${EndIf}
            IntOp $2 $2 + 1
        ${EndWhile}
    ${EndIf}
FunctionEnd

Function un.customUnGUIInit
    System::Call 'dwmapi::DwmSetWindowAttribute(p $HWNDPARENT, i 20, *i 1, i 4)'
    SetCtlColors $HWNDPARENT 0xE9EEF1 0x0F151A
FunctionEnd

Function un.unPageShow
    System::Call 'dwmapi::DwmSetWindowAttribute(p $HWNDPARENT, i 20, *i 1, i 4)'
    SetCtlColors $HWNDPARENT 0xE9EEF1 0x0F151A
    
    GetDlgItem $1 $HWNDPARENT 1028
    ${If} $1 != 0
        SetCtlColors $1 0x7E8C9A 0x0F151A
    ${EndIf}

    FindWindow $0 "#32770" "" $HWNDPARENT
    ${If} $0 != 0
        SetCtlColors $0 0xE9EEF1 0x0F151A
        
        StrCpy $2 1000
        ${While} $2 <= 1040
            GetDlgItem $1 $0 $2
            ${If} $1 != 0
                ${If} $2 == 1019
                    SetCtlColors $1 0xE9EEF1 0x1A2228
                ${Else}
                    SetCtlColors $1 0xE9EEF1 0x0F151A
                ${EndIf}
            ${EndIf}
            IntOp $2 $2 + 1
        ${EndWhile}
    ${EndIf}
FunctionEnd

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
