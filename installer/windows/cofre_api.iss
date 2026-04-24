#define MyAppName "CofreSenhaRust API"
#define MyAppExeName "cofre_api.exe"

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
Source: "register_task.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "unregister_task.ps1"; DestDir: "{app}"; Flags: ignoreversion

[Run]
Filename: "powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\register_task.ps1"" -ExePath ""{app}\cofre_api.exe"" -TaskName ""{#TaskName}"" -Port {#ApiPort} -SessionTtlSecs {#SessionTtlSecs}"; Flags: runhidden waituntilterminated

[UninstallRun]
Filename: "powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\unregister_task.ps1"" -TaskName ""{#TaskName}"""; Flags: runhidden waituntilterminated; RunOnceId: "UnregisterCofreApiTask"
