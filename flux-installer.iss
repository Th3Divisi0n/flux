; Flux Language Installer
; Built with Inno Setup

[Setup]
AppName=Flux
AppVersion=1.0.0
AppPublisher=Th3Divisi0n
AppPublisherURL=https://github.com/Th3Divisi0n/flux
DefaultDirName={autopf}\Flux
DefaultGroupName=Flux
AllowNoIcons=yes
OutputBaseFilename=FluxSetup
Compression=lzma2
SolidCompression=yes
ChangesEnvironment=yes
DisableProgramGroupPage=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "target\release\fx.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Flux"; Filename: "{app}\fx.exe"
Name: "{group}\Uninstall Flux"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Flux"; Filename: "{app}\fx.exe"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Flags: preservestringtype; Check: NeedsAddPath('{app}')

[Code]
function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', OrigPath) then
  begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + ExpandConstant('{app}') + ';', ';' + OrigPath + ';') = 0;
end;
