; AgentDesk Windows Installer — Inno Setup Script
; Copyright (c) 2025 Next Gen Innovations Private Limited
; Developer: Abhishek Kumar Paswan
; Website: https://nextgeninnovations.com.np/
; Made in Nepal

#define MyAppName "AgentDesk"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "Next Gen Innovations Private Limited"
#define MyAppURL "https://nextgeninnovations.com.np/"
#define MyAppExeName "agentdesk.exe"
#define MyAppDescription "Free & Open Source Remote Desktop — Made in Nepal"

[Setup]
; App identity & metadata
AppId={{8B7C3D2E-1A4F-4B6C-9D8E-2F5A7B9C1D3E}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
AppComments={#MyAppDescription}
AppReadmeFile=README.md

; Default installation directory (Windows convention)
; {autopf} = "C:\Program Files" for 64-bit installs
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}

; Allow user to choose per-user (no admin) or per-machine (admin) install
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

; Classic Next-Next-Finish wizard (not modern ribbon style)
WizardStyle=classic
DisableWelcomePage=no
DisableDirPage=no
DisableProgramGroupPage=no
DisableReadyPage=no
DisableFinishedPage=no

; License file display
LicenseFile=packaging\windows\LICENSE.txt

; Output settings
OutputDir=dist\windows
OutputBaseFilename=AgentDesk-Setup-{#MyAppVersion}
SetupIconFile=packaging\windows\agentdesk.ico

; Compression
Compression=lzma2/ultra64
SolidCompression=yes
CompressionThreads=auto

; Platform requirements
MinVersion=10.0
ArchitecturesInstallIn64BitMode=x64compatible

; Uninstaller configuration
Uninstallable=yes
UninstallDisplayName={#MyAppName}
UninstallDisplayIcon={app}\{#MyAppExeName}
CreateUninstallRegKey=yes

; Installer appearance
WizardImageFile=compiler:WizModernImage-IS.bmp
WizardSmallImageFile=compiler:WizModernSmallImage-IS.bmp

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[CustomMessages]
english.WelcomeLabel2=This will install [name/ver] on your computer.%n%nAgentDesk is a 100%% Free and Open Source Remote Desktop application — Made in Nepal by Next Gen Innovations Private Limited.%n%nNo time limits, no device limits, no account required, no telemetry.%n%nIt is recommended that you close all other applications before continuing.%n%nClick Next to continue, or Cancel to exit Setup.
english.FinishedHeadingLabel=Completing [name] Setup
english.FinishedLabel=[name] has been installed on your computer.%n%nThe application may be launched from your Start Menu or Desktop shortcut.%n%nThank you for using AgentDesk!

[Tasks]
Name: "desktopicon"; \
    Description: "Create a &Desktop shortcut"; \
    GroupDescription: "Additional icons:"; \
    Flags: checkedonce
Name: "startmenuicon"; \
    Description: "Create a &Start Menu shortcut"; \
    GroupDescription: "Additional icons:"; \
    Flags: checkedonce
Name: "startup"; \
    Description: "Start AgentDesk automatically with &Windows (not recommended)"; \
    GroupDescription: "Other:"; \
    Flags: unchecked

[Files]
; Main binary
Source: "target\release\agentdesk.exe"; \
    DestDir: "{app}"; \
    Flags: ignoreversion

; Documentation
Source: "LICENSE"; \
    DestDir: "{app}"; \
    DestName: "LICENSE.txt"; \
    Flags: ignoreversion
Source: "README.md"; \
    DestDir: "{app}"; \
    Flags: ignoreversion
Source: "docs\TESTING_GUIDE.md"; \
    DestDir: "{app}\docs"; \
    Flags: ignoreversion skipifsourcedoesntexist

[Icons]
; Start Menu shortcuts
Name: "{group}\{#MyAppName}"; \
    Filename: "{app}\{#MyAppExeName}"; \
    WorkingDir: "{app}"; \
    Tasks: startmenuicon

Name: "{group}\Uninstall {#MyAppName}"; \
    Filename: "{uninstallexe}"; \
    Tasks: startmenuicon

; Desktop shortcut (optional task)
Name: "{autodesktop}\{#MyAppName}"; \
    Filename: "{app}\{#MyAppExeName}"; \
    WorkingDir: "{app}"; \
    Tasks: desktopicon

[Registry]
; Add to Windows startup (optional task)
Root: HKCU; \
    Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; \
    ValueType: string; \
    ValueName: "AgentDesk"; \
    ValueData: """{app}\{#MyAppExeName}"""; \
    Flags: uninsdeletevalue; \
    Tasks: startup

; Register app capabilities in Windows (for Add/Remove Programs enrichment)
Root: HKCU; \
    Subkey: "Software\{#MyAppPublisher}\{#MyAppName}"; \
    ValueType: string; \
    ValueName: "InstallPath"; \
    ValueData: "{app}"; \
    Flags: uninsdeletekey

Root: HKCU; \
    Subkey: "Software\{#MyAppPublisher}\{#MyAppName}"; \
    ValueType: string; \
    ValueName: "Version"; \
    ValueData: "{#MyAppVersion}"

[Run]
; "Launch AgentDesk" checkbox on the last wizard page
Filename: "{app}\{#MyAppExeName}"; \
    Description: "Launch {#MyAppName} now"; \
    Flags: nowait postinstall skipifsilent unchecked

[UninstallDelete]
; Remove app data folder on uninstall
Type: dirifempty; Name: "{app}"

[Code]
// --- Custom wizard pages & logic ---

// Check if app is currently running before install
function InitializeSetup(): Boolean;
var
  ResultCode: Integer;
begin
  Result := True;
  if CheckForMutexes('AgentDesk-Running') then begin
    if MsgBox(
      'AgentDesk is currently running. It must be closed before installing.' + #13#10 +
      'Close AgentDesk and click OK, or click Cancel to abort installation.',
      mbConfirmation, MB_OKCANCEL) = IDCANCEL then
      Result := False;
  end;
end;

// Kill running process during update installs
procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
begin
  if CurStep = ssInstall then begin
    Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM agentdesk.exe', '', SW_HIDE,
         ewWaitUntilTerminated, ResultCode);
  end;
end;
