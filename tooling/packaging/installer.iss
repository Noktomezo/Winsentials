; Winsentials Inno Setup 6 Script
; Modern UI with High-DPI, 64-bit Architecture, and Solid LZMA2 Compression

#ifndef MyAppVersion
  #define MyAppVersion "0.1.0"
#endif

#ifndef MyOutputDir
  #define MyOutputDir "..\..\target\release"
#endif

#ifndef MyOutputBaseFilename
  #define MyOutputBaseFilename "winsentials-win-x64-setup"
#endif

#ifndef MyBinaryPath
  #define MyBinaryPath "..\..\target\release\Winsentials.exe"
#endif

[Setup]
AppId={{8B4119E1-EFE1-4E67-93C9-9A0B83E15993}
AppName=Winsentials
AppVersion={#MyAppVersion}
AppVerName=Winsentials {#MyAppVersion}
AppPublisher=Noktomezo
AppPublisherURL=https://github.com/Noktomezo/Winsentials
AppSupportURL=https://github.com/Noktomezo/Winsentials
AppUpdatesURL=https://github.com/Noktomezo/Winsentials
DefaultDirName={autopf64}\Winsentials
DefaultGroupName=Winsentials
DisableProgramGroupPage=yes
DisableDirPage=no
ShowLanguageDialog=auto
OutputDir={#MyOutputDir}
OutputBaseFilename={#MyOutputBaseFilename}
SetupIconFile=..\..\assets\app-logo.ico
UninstallDisplayIcon={app}\Winsentials.exe
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern dynamic
WizardSizePercent=100
WizardImageFile=..\..\assets\app-installer-sidebar.bmp
WizardSmallImageFile=..\..\assets\app-installer-small.bmp
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=admin
CloseApplications=yes
CloseApplicationsFilter=Winsentials.exe
RestartApplications=no

[Languages]
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "{#MyBinaryPath}"; DestDir: "{app}"; DestName: "Winsentials.exe"; Flags: ignoreversion

[Icons]
Name: "{group}\Winsentials"; Filename: "{app}\Winsentials.exe"
Name: "{group}\{cm:UninstallProgram,Winsentials}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Winsentials"; Filename: "{app}\Winsentials.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\Winsentials.exe"; Description: "{cm:LaunchProgram,Winsentials}"; Flags: nowait postinstall skipifsilent

[Code]
function DwmSetWindowAttribute(hWnd: HWND; dwAttribute: DWORD; var pvAttribute: DWORD; cbAttribute: DWORD): LongInt;
  external 'DwmSetWindowAttribute@dwmapi.dll stdcall';

procedure InitializeWizard;
var
  DarkModeVal: DWORD;
begin
  DarkModeVal := 1;
  // DWMWA_USE_IMMERSIVE_DARK_MODE = 20 (Windows 11 / Windows 10 20H1+)
  DwmSetWindowAttribute(WizardForm.Handle, 20, DarkModeVal, 4);
  DwmSetWindowAttribute(WizardForm.Handle, 19, DarkModeVal, 4);
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
begin
  if CurStep = ssInstall then
  begin
    Exec('taskkill.exe', '/F /IM Winsentials.exe /T', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  ResultCode: Integer;
begin
  if CurUninstallStep = usUninstall then
  begin
    Exec('taskkill.exe', '/F /IM Winsentials.exe /T', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  end;
end;