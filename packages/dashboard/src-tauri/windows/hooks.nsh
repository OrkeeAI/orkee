; ABOUTME: NSIS installer hooks for Windows Orkee desktop app
; ABOUTME: Adds orkee binary to system PATH during installation

; Include required libraries
!include "LogicLib.nsh"

; Constants
!define MAX_PATH_LENGTH 2047        ; Windows environment variable length limit
!define BROADCAST_TIMEOUT 10000     ; Timeout for WM_SETTINGCHANGE broadcast (milliseconds)

; Helper function to check if string contains substring
Function StrContains
  Exch 1
  Exch
  Exch $R0 ; input string
  Exch
  Exch $R1 ; substring to find
  Push $R2
  Push $R3

  StrLen $R3 $R1
  StrCpy $R2 0

  loop:
    StrCpy $R4 $R0 $R3 $R2
    StrCmp $R4 "" notfound
    StrCmp $R4 $R1 found
    IntOp $R2 $R2 + 1
    Goto loop

  found:
    StrCpy $R0 "1"
    Goto done

  notfound:
    StrCpy $R0 ""

  done:
  Pop $R3
  Pop $R2
  Pop $R1
  Exch $R0
FunctionEnd

!macro StrContains output substring input
  Push "${input}"
  Push "${substring}"
  Call StrContains
  Pop "${output}"
!macroend

!define StrContains "!insertmacro StrContains"

; Uninstall version of StrContains function
Function un.StrContains
  Exch 1
  Exch
  Exch $R0 ; input string
  Exch
  Exch $R1 ; substring to find
  Push $R2
  Push $R3

  StrLen $R3 $R1
  StrCpy $R2 0

  loop:
    StrCpy $R4 $R0 $R3 $R2
    StrCmp $R4 "" notfound
    StrCmp $R4 $R1 found
    IntOp $R2 $R2 + 1
    Goto loop

  found:
    StrCpy $R0 "1"
    Goto done

  notfound:
    StrCpy $R0 ""

  done:
  Pop $R3
  Pop $R2
  Pop $R1
  Exch $R0
FunctionEnd

!macro un.StrContains output substring input
  Push "${input}"
  Push "${substring}"
  Call un.StrContains
  Pop "${output}"
!macroend

!define un.StrContains "!insertmacro un.StrContains"

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
      ; Not found - add it
      StrCpy $2 "$2;$0"
      WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" "$2"
      ; Broadcast WM_SETTINGCHANGE to notify system of PATH change
      SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=${BROADCAST_TIMEOUT}
    ${EndIf}
  ${Else}
    ; Per-user: Add to user PATH (HKCU)
    ; No admin privileges needed

    ; Read current user PATH
    ReadRegStr $2 HKCU "Environment" "Path"

    ; Check if our directory is already in PATH
    ${StrContains} $3 "$0" "$2"
    ${If} $3 == ""
      ; Not found - add it
      StrCpy $2 "$2;$0"
      WriteRegExpandStr HKCU "Environment" "Path" "$2"
      ; Broadcast WM_SETTINGCHANGE to notify system of PATH change
      SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=${BROADCAST_TIMEOUT}
    ${EndIf}
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Detect installation mode by checking where the app was installed
  ${un.StrContains} $R0 "Program Files" "$INSTDIR"

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
    ${un.StrContains} $3 "$0" "$2"
    ${If} $3 != ""
      ; Remove the directory from PATH
      ; This is simplified - just removing our specific directory
      Push $2
      Push ";$0"
      Push ""
      Call un.StrReplace
      Pop $2

      Push $2
      Push "$0;"
      Push ""
      Call un.StrReplace
      Pop $2

      Push $2
      Push "$0"
      Push ""
      Call un.StrReplace
      Pop $2

      ; Write updated PATH back to registry
      WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" "$2"

      ; Broadcast WM_SETTINGCHANGE to notify system of PATH change
      SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=${BROADCAST_TIMEOUT}
    ${EndIf}
  ${Else}
    ; Per-user: Remove from user PATH (HKCU)
    ReadRegStr $2 HKCU "Environment" "Path"

    ; Check if our directory is in PATH
    ${un.StrContains} $3 "$0" "$2"
    ${If} $3 != ""
      ; Remove the directory from PATH
      Push $2
      Push ";$0"
      Push ""
      Call un.StrReplace
      Pop $2

      Push $2
      Push "$0;"
      Push ""
      Call un.StrReplace
      Pop $2

      Push $2
      Push "$0"
      Push ""
      Call un.StrReplace
      Pop $2

      ; Write updated PATH back to registry
      WriteRegExpandStr HKCU "Environment" "Path" "$2"

      ; Broadcast WM_SETTINGCHANGE to notify system of PATH change
      SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=${BROADCAST_TIMEOUT}
    ${EndIf}
  ${EndIf}
!macroend

; String replace function
Function StrReplace
  Exch $R4 ; replacement
  Exch
  Exch $R3 ; string to replace
  Exch
  Exch 2
  Exch $R1 ; input string
  Push $R2
  Push $R5
  Push $R6
  Push $R7

  StrCpy $R2 ""
  StrLen $R5 $R1
  StrLen $R6 $R3
  StrLen $R7 $R4

  Loop:
    StrCpy $R0 $R1 $R6
    StrCmp $R0 $R3 Replace
    StrCmp $R0 "" Done
    StrCpy $R0 $R1 1
    StrCpy $R2 "$R2$R0"
    StrCpy $R1 $R1 "" 1
    Goto Loop

  Replace:
    StrCpy $R2 "$R2$R4"
    StrCpy $R1 $R1 "" $R6
    Goto Loop

  Done:
    StrCpy $R1 "$R2$R1"
    Pop $R7
    Pop $R6
    Pop $R5
    Pop $R2
    Pop $R0
    Pop $R4
    Pop $R3
    Exch $R1
FunctionEnd

; Uninstall version of StrReplace function
Function un.StrReplace
  Exch $R4 ; replacement
  Exch
  Exch $R3 ; string to replace
  Exch
  Exch 2
  Exch $R1 ; input string
  Push $R2
  Push $R5
  Push $R6
  Push $R7

  StrCpy $R2 ""
  StrLen $R5 $R1
  StrLen $R6 $R3
  StrLen $R7 $R4

  Loop:
    StrCpy $R0 $R1 $R6
    StrCmp $R0 $R3 Replace
    StrCmp $R0 "" Done
    StrCpy $R0 $R1 1
    StrCpy $R2 "$R2$R0"
    StrCpy $R1 $R1 "" 1
    Goto Loop

  Replace:
    StrCpy $R2 "$R2$R4"
    StrCpy $R1 $R1 "" $R6
    Goto Loop

  Done:
    StrCpy $R1 "$R2$R1"
    Pop $R7
    Pop $R6
    Pop $R5
    Pop $R2
    Pop $R0
    Pop $R4
    Pop $R3
    Exch $R1
FunctionEnd