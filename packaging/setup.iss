#define AppName "OpenLogi Lighting Hotkey"
#define AppVersion "0.1.0"
#define AppExe "openlogi-lighting-hotkey.exe"

[Setup]
AppId={{F8DE6BD7-B228-4CBB-9F9E-49035672DEAA}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=mynameistito
AppPublisherURL=https://github.com/mynameistito/openlogi-lighting-hotkey
AppSupportURL=https://github.com/mynameistito/openlogi-lighting-hotkey/issues
DefaultDirName={localappdata}\OpenLogiLightingHotkey
DefaultGroupName={#AppName}
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\dist
OutputBaseFilename=openlogi-lighting-hotkey-setup-{#AppVersion}
LicenseFile=..\LICENSE
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\{#AppExe}
CloseApplications=yes
RestartApplications=no
DisableProgramGroupPage=yes

[Tasks]
Name: startup; Description: "Start automatically when I sign in to Windows"; GroupDescription: "Startup:";
Name: desktopicon; Description: "Create a desktop shortcut"; GroupDescription: "Shortcuts:";

[Files]
Source: "..\dist\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\settings.toml"; DestDir: "{app}"; Flags: onlyifdoesntexist uninsneveruninstall
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExe}"; WorkingDir: "{app}"
Name: "{userdesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; WorkingDir: "{app}"; Tasks: desktopicon
Name: "{userstartup}\{#AppName}"; Filename: "{app}\{#AppExe}"; WorkingDir: "{app}"; Tasks: startup
Name: "{group}\Edit settings"; Filename: "{sys}\notepad.exe"; Parameters: """{app}\settings.toml"""
Name: "{group}\Uninstall {#AppName}"; Filename: "{uninstallexe}"

[Run]
Filename: "{app}\{#AppExe}"; Description: "Start {#AppName}"; Flags: nowait postinstall skipifsilent
