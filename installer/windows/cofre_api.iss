#define MyAppName "CofreSenhaRust API"
#define MyAppExeName "cofre_api.exe"
#define MyTrayExeName "cofre_tray.exe"

#ifndef MyAppVersion
  #define MyAppVersion "0.1.0"
#endif

#ifndef ApiPort
  #define ApiPort "5474"
#endif

#ifndef SessionTtlSecs
  #define SessionTtlSecs "1800"
#endif

#ifndef TaskName
  #define TaskName "CofreApi"
#endif

[Setup]
AppId={{9DCA0F1E-2C66-4F49-8587-F95E48E8C67B}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
DefaultDirName={localappdata}\CofreSenhaRust\api
DisableProgramGroupPage=yes
OutputDir=..\..\dist\windows
OutputBaseFilename=CofreSenhaRustApi-Setup-{#MyAppVersion}
Compression=lzma
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ArchitecturesInstallIn64BitMode=x64compatible

[Files]
Source: "..\..\target\release\cofre_api.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\target\release\cofre_tray.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "unregister_task.ps1"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "{#TaskName}"; ValueData: """{app}\{#MyTrayExeName}"""; Flags: uninsdeletevalue

[Run]
Filename: "powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\unregister_task.ps1"" -TaskName ""{#TaskName}"""; Flags: runhidden waituntilterminated
Filename: "{app}\{#MyTrayExeName}"; Flags: runhidden nowait

[UninstallRun]
Filename: "powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\unregister_task.ps1"" -TaskName ""{#TaskName}"""; Flags: runhidden waituntilterminated; RunOnceId: "UnregisterCofreApiTask"
Filename: "powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -Command ""Stop-Process -Name cofre_tray,cofre_api -Force -ErrorAction SilentlyContinue"""; Flags: runhidden waituntilterminated; RunOnceId: "StopCofreApiTray"
