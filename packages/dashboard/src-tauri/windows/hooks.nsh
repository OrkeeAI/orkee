; ABOUTME: NSIS installer hooks for Windows Orkee desktop app
; ABOUTME: Adds orkee binary to system PATH during installation

; Include required string function libraries
!include "StrFunc.nsh"
!include "WordFunc.nsh"

; Constants
!define MAX_PATH_LENGTH 2047        ; Windows environment variable length limit
!define BROADCAST_TIMEOUT 10000     ; Timeout for WM_SETTINGCHANGE broadcast (milliseconds)

!macro NSIS_HOOK_POSTINSTALL
  ; Copy bundled orkee.exe to a stable location
  ; The sidecar binary is embedded in the app, but we want it accessible from terminal

  ; Detect installation mode by checking where the app was installed
  ; $INSTDIR is set by Tauri based on the installMode configuration
  ; perMachine: $PROGRAMFILES or $PROGRAMFILES64
  ; perUser/currentUser: $LOCALAPPDATA

  ; Check if this is a per-machine install by looking for Program Files in path
  ${StrContains} $R0 "Program Files" "$INSTDIR"

  ${If} $R0 != ""
    ; System-wide install (perMachine): use Program Files
    StrCpy $0 "$PROGRAMFILES\Orkee\bin"
    StrCpy $1 "perMachine"
  ${Else}
    ; Per-user install: use Local AppData
    StrCpy $0 "$LOCALAPPDATA\Orkee\bin"
    StrCpy $1 "perUser"
  ${EndIf}

  ; Create bin directory if it doesn't exist
  CreateDirectory "$0"

  ; Copy orkee.exe from app resources to bin directory
  ; The binary is bundled as a sidecar in the app
  CopyFiles /SILENT "$INSTDIR\orkee.exe" "$0\orkee.exe"

  ; Add to PATH based on install mode (only if not already present)
  ${If} $1 == "perMachine"
    ; System-wide: Add to system PATH (HKLM)
    ; This requires admin privileges

    ; Read current system PATH
    ReadRegStr $2 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path"

    ; Check if our directory is already in PATH
    ${StrContains} $3 "$0" "$2"
    ${If} $3 == ""
      ; Not found - check PATH length before adding
      StrLen $4 "$2;$0"
      ${If} $4 < ${MAX_PATH_LENGTH}
        ; Safe to add - under Windows PATH limit
        WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" "$2;$0"
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
    ReadRegStr $2 HKCU "Environment" "Path"

    ; Check if our directory is already in PATH
    ${StrContains} $3 "$0" "$2"
    ${If} $3 == ""
      ; Not found - check PATH length before adding
      StrLen $4 "$2;$0"
      ${If} $4 < ${MAX_PATH_LENGTH}
        ; Safe to add - under Windows PATH limit
        WriteRegExpandStr HKCU "Environment" "Path" "$2;$0"
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
  ; Detect installation mode by checking where the app was installed
  ${StrContains} $R0 "Program Files" "$INSTDIR"

  ${If} $R0 != ""
    ; System-wide install: use Program Files
    StrCpy $0 "$PROGRAMFILES\Orkee\bin"
    StrCpy $1 "perMachine"
  ${Else}
    ; Per-user install: use Local AppData
    StrCpy $0 "$LOCALAPPDATA\Orkee\bin"
    StrCpy $1 "perUser"
  ${EndIf}

  ; Delete binary
  Delete "$0\orkee.exe"

  ; Remove bin directory if empty
  RMDir "$0"

  ; Clean up PATH entries
  ${If} $1 == "perMachine"
    ; System-wide: Remove from system PATH (HKLM)
    ReadRegStr $2 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path"

    ; Check if our directory is in PATH
    ${StrContains} $3 "$0" "$2"
    ${If} $3 != ""
      ; Remove the directory from PATH (try all possible positions)
      ${WordReplace} $2 ";$0" "" "+" $2
      ${WordReplace} $2 "$0;" "" "+" $2
      ${WordReplace} $2 "$0" "" "+" $2

      ; Write updated PATH back to registry
      WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" "$2"

      ; Broadcast WM_SETTINGCHANGE to notify system of PATH change
      SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=${BROADCAST_TIMEOUT}
    ${EndIf}
  ${Else}
    ; Per-user: Remove from user PATH (HKCU)
    ReadRegStr $2 HKCU "Environment" "Path"

    ; Check if our directory is in PATH
    ${StrContains} $3 "$0" "$2"
    ${If} $3 != ""
      ; Remove the directory from PATH (try all possible positions)
      ${WordReplace} $2 ";$0" "" "+" $2
      ${WordReplace} $2 "$0;" "" "+" $2
      ${WordReplace} $2 "$0" "" "+" $2

      ; Write updated PATH back to registry
      WriteRegExpandStr HKCU "Environment" "Path" "$2"

      ; Broadcast WM_SETTINGCHANGE to notify system of PATH change
      SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=${BROADCAST_TIMEOUT}
    ${EndIf}
  ${EndIf}
!macroend
