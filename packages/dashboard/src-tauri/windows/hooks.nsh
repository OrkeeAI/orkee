; ABOUTME: NSIS installer hooks for Windows Orkee desktop app
; ABOUTME: Adds orkee binary to system PATH during installation

; Constants
!define MAX_PATH_LENGTH 2047        ; Windows environment variable length limit
!define BROADCAST_TIMEOUT 10000     ; Timeout for WM_SETTINGCHANGE broadcast (milliseconds)

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

  ; Add to PATH based on install mode (only if not already present)
  ${If} $R0 == "perMachine"
    ; System-wide: Add to system PATH (HKLM)
    ; This requires admin privileges

    ; Read current system PATH
    ReadRegStr $1 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path"

    ; Check if our directory is already in PATH
    ${StrContains} $2 "$0" "$1"
    ${If} $2 == ""
      ; Not found - check PATH length before adding
      StrLen $3 "$1;$0"
      ${If} $3 < ${MAX_PATH_LENGTH}
        ; Safe to add - under Windows PATH limit
        WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" "$1;$0"
        ; Broadcast WM_SETTINGCHANGE to notify system of PATH change
        SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=${BROADCAST_TIMEOUT}
      ${Else}
        ; PATH too long - warn user but don't fail installation
        MessageBox MB_OK|MB_ICONEXCLAMATION "Warning: System PATH is too long to add Orkee automatically.$\n$\nYou can manually add this directory to PATH:$\n$0$\n$\nDesktop app will still work without CLI access."
      ${EndIf}
    ${EndIf}
  ${Else}
    ; Per-user: Add to user PATH (HKCU)
    ; No admin privileges needed

    ; Read current user PATH
    ReadRegStr $1 HKCU "Environment" "Path"

    ; Check if our directory is already in PATH
    ${StrContains} $2 "$0" "$1"
    ${If} $2 == ""
      ; Not found - check PATH length before adding
      StrLen $3 "$1;$0"
      ${If} $3 < ${MAX_PATH_LENGTH}
        ; Safe to add - under Windows PATH limit
        WriteRegExpandStr HKCU "Environment" "Path" "$1;$0"
        ; Broadcast WM_SETTINGCHANGE to notify system of PATH change
        SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=${BROADCAST_TIMEOUT}
      ${Else}
        ; PATH too long - warn user but don't fail installation
        MessageBox MB_OK|MB_ICONEXCLAMATION "Warning: User PATH is too long to add Orkee automatically.$\n$\nYou can manually add this directory to PATH:$\n$0$\n$\nDesktop app will still work without CLI access."
      ${EndIf}
    ${EndIf}
  ${EndIf}
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

  ; Clean up PATH entries
  ${If} $R0 == "perMachine"
    ; System-wide: Remove from system PATH (HKLM)
    ReadRegStr $1 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path"

    ; Check if our directory is in PATH
    ${StrContains} $2 "$0" "$1"
    ${If} $2 != ""
      ; Remove the directory from PATH
      ${WordReplace} $1 ";$0" "" "+" $1
      ${WordReplace} $1 "$0;" "" "+" $1
      ${WordReplace} $1 "$0" "" "+" $1

      ; Write updated PATH back to registry
      WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" "$1"

      ; Broadcast WM_SETTINGCHANGE to notify system of PATH change
      SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=${BROADCAST_TIMEOUT}
    ${EndIf}
  ${Else}
    ; Per-user: Remove from user PATH (HKCU)
    ReadRegStr $1 HKCU "Environment" "Path"

    ; Check if our directory is in PATH
    ${StrContains} $2 "$0" "$1"
    ${If} $2 != ""
      ; Remove the directory from PATH
      ${WordReplace} $1 ";$0" "" "+" $1
      ${WordReplace} $1 "$0;" "" "+" $1
      ${WordReplace} $1 "$0" "" "+" $1

      ; Write updated PATH back to registry
      WriteRegExpandStr HKCU "Environment" "Path" "$1"

      ; Broadcast WM_SETTINGCHANGE to notify system of PATH change
      SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=${BROADCAST_TIMEOUT}
    ${EndIf}
  ${EndIf}
!macroend
