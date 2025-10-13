; ABOUTME: NSIS installer hooks for Windows Orkee desktop app
; ABOUTME: Adds orkee binary to system PATH during installation

!macro NSIS_HOOK_POSTINSTALL
  ; Copy bundled orkee.exe to a stable location
  ; The sidecar binary is embedded in the app, but we want it accessible from terminal

  ; Determine installation directory based on install mode
  ${If} $R0 == "perMachine"
    ; System-wide install: use Program Files
    StrCpy $0 "$PROGRAMFILES\Orkee\bin"
  ${Else}
    ; Per-user install: use Local AppData
    StrCpy $0 "$LOCALAPPDATA\Orkee\bin"
  ${EndIf}

  ; Create bin directory if it doesn't exist
  CreateDirectory "$0"

  ; Copy orkee.exe from app resources to bin directory
  ; The binary is bundled as a sidecar in the app
  CopyFiles /SILENT "$INSTDIR\orkee.exe" "$0\orkee.exe"

  ; Add to PATH based on install mode
  ${If} $R0 == "perMachine"
    ; System-wide: Add to system PATH (HKLM)
    ; This requires admin privileges
    nsExec::ExecToLog 'setx /M PATH "$0;%PATH%"'
  ${Else}
    ; Per-user: Add to user PATH (HKCU)
    ; No admin privileges needed
    nsExec::ExecToLog 'setx PATH "$0;%PATH%"'
  ${EndIf}

  ; Notify system of environment variable change
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Remove orkee binary and bin directory
  ${If} $R0 == "perMachine"
    StrCpy $0 "$PROGRAMFILES\Orkee\bin"
  ${Else}
    StrCpy $0 "$LOCALAPPDATA\Orkee\bin"
  ${EndIf}

  ; Delete binary
  Delete "$0\orkee.exe"

  ; Remove bin directory if empty
  RMDir "$0"

  ; Note: We intentionally don't remove from PATH to avoid breaking
  ; user's environment if they have other Orkee installations.
  ; The PATH entry won't cause issues after uninstall since the
  ; directory will be deleted.
!macroend
