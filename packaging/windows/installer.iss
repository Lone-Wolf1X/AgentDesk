; Inno Setup Script for AgentDesk
; Free & Open Source Remote Desktop — Made in Nepal
; Developed by Next Gen Innovations Private Limited & Abhishek Kumar Paswan

#define MyAppName "AgentDesk"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "Next Gen Innovations Private Limited"
#define MyAppURL "https://nextgeninnovations.com.np/"
#define MyAppExeName "agentdesk-viewer.exe"
#define MyHostExeName "agentdesk-host.exe"

[Setup]
AppId={{D1A2E8B9-6C43-4F81-9E45-8B4F9A6C23D1}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DisableProgramGroupPage=yes
LicenseFile=..\..\LICENSE
OutputDir=Output
OutputBaseFilename=AgentDesk-Setup-{#MyAppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\target\release\{#MyHostExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\AgentDesk Viewer"; Filename: "{app}\{#MyAppExeName}"
Name: "{autoprograms}\AgentDesk Host"; Filename: "{app}\{#MyHostExeName}"
Name: "{autodesktop}\AgentDesk Viewer"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon
Name: "{autodesktop}\AgentDesk Host"; Filename: "{app}\{#MyHostExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,AgentDesk Viewer}"; Flags: nowait postinstall skipifsilent
