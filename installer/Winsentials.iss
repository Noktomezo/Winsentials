#ifndef AppVersion
  #define AppVersion "0.10.0"
#endif
#ifndef SourceExe
  #define SourceExe "..\target\release\Winsentials.exe"
#endif
#ifndef SetupIcon
  #define SetupIcon "..\assets\app-logo.ico"
#endif

[Setup]
AppId={{D1E2FD03-6703-4F29-8D0D-E948B4D34CB2}
AppName=Winsentials
AppVersion={#AppVersion}
AppPublisher=Noktomezo
AppPublisherURL=https://github.com/Noktomezo/Winsentials
AppSupportURL=https://github.com/Noktomezo/Winsentials/issues
AppUpdatesURL=https://github.com/Noktomezo/Winsentials/releases
DefaultDirName={autopf}\Winsentials
DefaultGroupName=Winsentials
DisableProgramGroupPage=yes
OutputBaseFilename=winsentials-win-x64-setup
Compression=lzma2/ultra64
SolidCompression=yes
SetupIconFile={#SetupIcon}
UninstallDisplayIcon={app}\Winsentials.exe
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern dynamic
ShowLanguageDialog=auto
UsePreviousLanguage=yes
UsePreviousTasks=yes
UsePreviousAppDir=yes
CloseApplications=yes
CloseApplicationsFilter=Winsentials.exe
RestartApplications=no
SetupLogging=yes
VersionInfoVersion={#AppVersion}
VersionInfoCompany=Noktomezo
VersionInfoDescription=Winsentials Installer
VersionInfoProductName=Winsentials
VersionInfoProductVersion={#AppVersion}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; DestName: "Winsentials.exe"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Winsentials"; Filename: "{app}\Winsentials.exe"
Name: "{autodesktop}\Winsentials"; Filename: "{app}\Winsentials.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\Winsentials.exe"; Description: "{cm:LaunchProgram,Winsentials}"; Flags: nowait postinstall skipifsilent
Filename: "{app}\Winsentials.exe"; Flags: nowait; Check: WizardSilent
