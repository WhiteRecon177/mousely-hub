Unicode true
ManifestDPIAware true
; Add in `dpiAwareness` `PerMonitorV2` to manifest for Windows 10 1607+ (note this should not affect lower versions since they should be able to ignore this and pick up `dpiAware` `true` set by `ManifestDPIAware true`)
; Currently undocumented on NSIS's website but is in the Docs folder of source tree, see
; https://github.com/kichik/nsis/blob/5fc0b87b819a9eec006df4967d08e522ddd651c9/Docs/src/attributes.but#L286-L300
; https://github.com/tauri-apps/tauri/pull/10106
ManifestDPIAwareness PerMonitorV2

!if "{{compression}}" == "none"
  SetCompress off
!else
  ; Set the compression algorithm. We default to LZMA.
  SetCompressor /SOLID "{{compression}}"
!endif

; Keep above !include to stay ahead of any plugin command
; see https://github.com/tauri-apps/tauri/pull/15422#discussion_r3289239624
{{#if signed_plugins_path}}
!addplugindir "{{signed_plugins_path}}"
{{/if}}

!include MUI2.nsh
!include FileFunc.nsh
!include x64.nsh
!include WordFunc.nsh
!include "utils.nsh"
!include "FileAssociation.nsh"
!include "Win\COM.nsh"
!include "Win\Propkey.nsh"
!include "Win\RestartManager.nsh"
!include "StrFunc.nsh"
${StrCase}
${StrLoc}

{{#if installer_hooks}}
!include "{{installer_hooks}}"
{{/if}}

; ===== Mousely Hub dark installer: inserted into Tauri's NSIS template by make-dark-installer.ps1 =====
!include LogicLib.nsh

!define DK_TEXT "EAF0FF"
!define DK_BG "070B1C"
!define DK_BG2 "131C45"
; DWM wants COLORREF = 0x00BBGGRR
!define DK_BGREF 0x001C0B07
!define DK_TXTREF 0x00FFF0EA

!define MUI_CUSTOMFUNCTION_GUIINIT DarkGuiInit
!define MUI_CUSTOMFUNCTION_UNGUIINIT un.DarkGuiInit
!define MUI_INSTFILESPAGE_COLORS "${DK_TEXT} ${DK_BG}"

!macro DK_DEFINE UN
; Recolors a window and its direct children. In: top of the stack = window handle.
Function ${UN}DarkPaint
  Exch $0
  Push $1
  Push $2
  Push $3
  SetCtlColors $0 "${DK_TEXT}" "${DK_BG}"
  StrCpy $1 0
  dk_loop:
    FindWindow $1 "" "" $0 $1
    IntCmp $1 0 dk_done
    System::Call 'user32::GetClassName(p r1, t .r2, i 40)'
    StrCmp $2 "#32770" dk_loop
    StrCmp $2 "msctls_progress32" dk_prog
    StrCmp $2 "Edit" dk_edit
    StrCmp $2 "Button" dk_btn
    StrCmp $2 "SysListView32" dk_list
    StrCmp $2 "RichEdit20W" dk_list
    SetCtlColors $1 "${DK_TEXT}" "${DK_BG}"
    Goto dk_loop
  dk_prog:
    System::Call 'uxtheme::SetWindowTheme(p r1, w "", w "")'
    SendMessage $1 0x409 0 0x00F6823B
    SendMessage $1 0x2001 0 0x0052251B
    Goto dk_loop
  dk_edit:
    SetCtlColors $1 "${DK_TEXT}" "${DK_BG2}"
    System::Call 'uxtheme::SetWindowTheme(p r1, w "DarkMode_Explorer", w n)'
    Goto dk_loop
  dk_list:
    System::Call 'uxtheme::SetWindowTheme(p r1, w "DarkMode_Explorer", w n)'
    Goto dk_loop
  dk_btn:
    System::Call 'user32::GetWindowLong(p r1, i -16) i .r3'
    IntOp $3 $3 & 15
    ${If} $3 < 2
      System::Call 'uxtheme::SetWindowTheme(p r1, w "DarkMode_Explorer", w n)'
    ${Else}
      System::Call 'uxtheme::SetWindowTheme(p r1, w "", w "")'
      SetCtlColors $1 "${DK_TEXT}" "${DK_BG}"
    ${EndIf}
    Goto dk_loop
  dk_done:
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

; Called when a page is shown (and for the custom "already installed" page).
Function ${UN}DarkPageShow
  Push $0
  FindWindow $0 "#32770" "" $HWNDPARENT
  Push $0
  Call ${UN}DarkPaint
  Pop $0
FunctionEnd

; Called once when the window opens: dark title bar (Windows 10 2004+ / 11), dark frame.
Function ${UN}DarkGuiInit
  System::Call 'dwmapi::DwmSetWindowAttribute(p $HWNDPARENT, i 20, *i 1, i 4)'
  System::Call 'dwmapi::DwmSetWindowAttribute(p $HWNDPARENT, i 35, *i ${DK_BGREF}, i 4)'
  System::Call 'dwmapi::DwmSetWindowAttribute(p $HWNDPARENT, i 36, *i ${DK_TXTREF}, i 4)'
  System::Call 'dwmapi::DwmSetWindowAttribute(p $HWNDPARENT, i 34, *i ${DK_BGREF}, i 4)'
  Push $HWNDPARENT
  Call ${UN}DarkPaint
  Call ${UN}SkChrome
FunctionEnd
!macroend
!insertmacro DK_DEFINE ""
!insertmacro DK_DEFINE "un."
; ===== end of dark installer block =====


!define WEBVIEW2APPGUID "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"

!define MANUFACTURER "{{manufacturer}}"
!define PRODUCTNAME "{{product_name}}"
!define VERSION "{{version}}"
!define VERSIONWITHBUILD "{{version_with_build}}"
!define HOMEPAGE "{{homepage}}"
!define INSTALLMODE "{{install_mode}}"
!define LICENSE "{{license}}"
!define INSTALLERICON "{{installer_icon}}"
!define SIDEBARIMAGE "{{sidebar_image}}"
!define HEADERIMAGE "{{header_image}}"
!define UNINSTALLERICON "{{uninstaller_icon}}"
!define UNINSTALLERHEADERIMAGE "{{uninstaller_header_image}}"
!define MAINBINARYNAME "{{main_binary_name}}"
!define MAINBINARYSRCPATH "{{main_binary_path}}"
!define BUNDLEID "{{bundle_id}}"
!define COPYRIGHT "{{copyright}}"
!define OUTFILE "{{out_file}}"
!define ARCH "{{arch}}"
!define ADDITIONALPLUGINSPATH "{{additional_plugins_path}}"
!define ALLOWDOWNGRADES "{{allow_downgrades}}"
!define DISPLAYLANGUAGESELECTOR "{{display_language_selector}}"
!define INSTALLWEBVIEW2MODE "{{install_webview2_mode}}"
!define WEBVIEW2INSTALLERARGS "{{webview2_installer_args}}"
!define WEBVIEW2BOOTSTRAPPERPATH "{{webview2_bootstrapper_path}}"
!define WEBVIEW2INSTALLERPATH "{{webview2_installer_path}}"
!define MINIMUMWEBVIEW2VERSION "{{minimum_webview2_version}}"
!define UNINSTKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCTNAME}"
!define MANUKEY "Software\${MANUFACTURER}"
!define MANUPRODUCTKEY "${MANUKEY}\${PRODUCTNAME}"
!define UNINSTALLERSIGNCOMMAND "{{uninstaller_sign_cmd}}"
!define ESTIMATEDSIZE "{{estimated_size}}"
!define STARTMENUFOLDER "{{start_menu_folder}}"

Var PassiveMode
Var UpdateMode
Var NoShortcutMode
Var WixMode
Var OldMainBinaryName

Name "${PRODUCTNAME}"
BrandingText "${COPYRIGHT}"
OutFile "${OUTFILE}"

; We don't actually use this value as default install path,
; it's just for nsis to append the product name folder in the directory selector
; https://nsis.sourceforge.io/Reference/InstallDir
!define PLACEHOLDER_INSTALL_DIR "placeholder\${PRODUCTNAME}"
InstallDir "${PLACEHOLDER_INSTALL_DIR}"

VIProductVersion "${VERSIONWITHBUILD}"
VIAddVersionKey "ProductName" "${PRODUCTNAME}"
VIAddVersionKey "FileDescription" "${PRODUCTNAME}"
VIAddVersionKey "LegalCopyright" "${COPYRIGHT}"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "ProductVersion" "${VERSION}"

# additional plugins
!addplugindir "${ADDITIONALPLUGINSPATH}"

; Uninstaller signing command
!if "${UNINSTALLERSIGNCOMMAND}" != ""
  !uninstfinalize '${UNINSTALLERSIGNCOMMAND}'
!endif

; Handle install mode, `perUser`, `perMachine` or `both`
!if "${INSTALLMODE}" == "perMachine"
  RequestExecutionLevel admin
!endif

!if "${INSTALLMODE}" == "currentUser"
  RequestExecutionLevel user
!endif

!if "${INSTALLMODE}" == "both"
  !define MULTIUSER_MUI
  !define MULTIUSER_INSTALLMODE_INSTDIR "${PRODUCTNAME}"
  !define MULTIUSER_INSTALLMODE_COMMANDLINE
  !if "${ARCH}" == "x64"
    !define MULTIUSER_USE_PROGRAMFILES64
  !else if "${ARCH}" == "arm64"
    !define MULTIUSER_USE_PROGRAMFILES64
  !endif
  !define MULTIUSER_INSTALLMODE_DEFAULT_REGISTRY_KEY "${UNINSTKEY}"
  !define MULTIUSER_INSTALLMODE_DEFAULT_REGISTRY_VALUENAME "CurrentUser"
  !define MULTIUSER_INSTALLMODEPAGE_SHOWUSERNAME
  !define MULTIUSER_INSTALLMODE_FUNCTION RestorePreviousInstallLocation
  !define MULTIUSER_EXECUTIONLEVEL Highest
  !include MultiUser.nsh
!endif

; Installer icon
!if "${INSTALLERICON}" != ""
  !define MUI_ICON "${INSTALLERICON}"
!endif

; Installer sidebar image
!if "${SIDEBARIMAGE}" != ""
  !define MUI_WELCOMEFINISHPAGE_BITMAP "${SIDEBARIMAGE}"
!endif

; Enable header images for installer and uninstaller pages when either image is configured.
!if "${HEADERIMAGE}" != ""
  !define MUI_HEADERIMAGE
!else if "${UNINSTALLERHEADERIMAGE}" != ""
  !define MUI_HEADERIMAGE
!endif

; Installer header image
!if "${HEADERIMAGE}" != ""
  !define MUI_HEADERIMAGE_BITMAP "${HEADERIMAGE}"
!endif

; Uninstaller header image
!if "${UNINSTALLERHEADERIMAGE}" != ""
  !define MUI_HEADERIMAGE_UNBITMAP "${UNINSTALLERHEADERIMAGE}"
!endif

; Uninstaller icon
!if "${UNINSTALLERICON}" != ""
  !define MUI_UNICON "${UNINSTALLERICON}"
!endif

; Define registry key to store installer language
!define MUI_LANGDLL_REGISTRY_ROOT "HKCU"
!define MUI_LANGDLL_REGISTRY_KEY "${MANUPRODUCTKEY}"
!define MUI_LANGDLL_REGISTRY_VALUENAME "Installer Language"

; ===== Mousely Hub custom installer skin: pages drawn as images, controls placed on top =====
!searchreplace SK_DIR "${SIDEBARIMAGE}" "sidebar.bmp" ""
!define SK_W 760
!define SK_H 500

Var SkW
Var SkH
Var SkBg
Var SkPfx
Var SkImg
Var SkReady
Var SkDlg
Var SkInst
Var SkDir
Var SkPct
Var SkVar
Var SkRun
Var SkProg
Var SkC1
Var SkC2
Var SkCk

!macro SK_FILE NAME
  File "/oname=$PLUGINSDIR\${NAME}.bmp" "${SK_DIR}${NAME}.bmp"
!macroend

; Moves a control to a rectangle given in design pixels (760x500), scaled to the window.
; Leaves the scaled width/height in $8/$9. Clobbers $6-$9.
!macro SK_PLACE HW X Y W H
  IntOp $6 ${X} * $SkW
  IntOp $6 $6 / ${SK_W}
  IntOp $7 ${Y} * $SkH
  IntOp $7 $7 / ${SK_H}
  IntOp $8 ${W} * $SkW
  IntOp $8 $8 / ${SK_W}
  IntOp $9 ${H} * $SkH
  IntOp $9 $9 / ${SK_H}
  System::Call 'user32::MoveWindow(p ${HW},i r6,i r7,i r8,i r9,i 1)'
!macroend

; Clickable picture (a crop of the page image): bitmap static with SS_NOTIFY, so it paints itself.
!macro SK_BTNX VAR X Y W H FUNC
  nsDialogs::CreateControl STATIC 0x5400010E 0 0 0 10u 10u ""
  Pop ${VAR}
  !insertmacro SK_PLACE ${VAR} ${X} ${Y} ${W} ${H}
  ${NSD_OnClick} ${VAR} ${FUNC}
!macroend
!macro SK_BTN VAR X Y W H FUNC FILE
  !insertmacro SK_BTNX ${VAR} ${X} ${Y} ${W} ${H} ${FUNC}
  Push ${VAR}
  Push "$SkImg\${FILE}-$SkPfx.bmp"
  Call SkSetCtlImg
!macroend

; Full-page background image: created, then pushed to the very bottom of the z-order.
!macro SK_BG FILE
  ${NSD_CreateBitmap} 0 0 100% 100% ""
  Pop $SkBg
  Push $SkBg
  Push "$SkImg\${FILE}-$SkPfx.bmp"
  Call SkSetImg
!macroend
!macro SK_BOTTOM
  System::Call 'user32::SetWindowPos(p $SkBg,p 1,i 0,i 0,i 0,i 0,i 0x13)'
!macroend

Function SkExtract
  InitPluginsDir
  StrCpy $SkImg $PLUGINSDIR
  !insertmacro SK_FILE chk
  !insertmacro SK_FILE p1-en
  !insertmacro SK_FILE p2-en
  !insertmacro SK_FILE p3-en
  !insertmacro SK_FILE p4s-a-en
  !insertmacro SK_FILE p4s-b-en
  !insertmacro SK_FILE p4u-a-en
  !insertmacro SK_FILE p4u-b-en
  !insertmacro SK_FILE p5-a-en
  !insertmacro SK_FILE p5-b-en
  !insertmacro SK_FILE c-p1-b-cancel-en
  !insertmacro SK_FILE c-p1-b-next-en
  !insertmacro SK_FILE c-p2-b-back-en
  !insertmacro SK_FILE c-p2-b-browse-en
  !insertmacro SK_FILE c-p2-b-install-en
  !insertmacro SK_FILE c-p5-b-fin-en
  !insertmacro SK_FILE c-p4s-b-cancel-en
  !insertmacro SK_FILE c-p4s-b-next-en
  !insertmacro SK_FILE c-p4s-card1-a-en
  !insertmacro SK_FILE c-p4s-card1-b-en
  !insertmacro SK_FILE c-p4s-card2-a-en
  !insertmacro SK_FILE c-p4s-card2-b-en
  !insertmacro SK_FILE c-p4u-b-cancel-en
  !insertmacro SK_FILE c-p4u-b-next-en
  !insertmacro SK_FILE c-p4u-card1-a-en
  !insertmacro SK_FILE c-p4u-card1-b-en
  !insertmacro SK_FILE c-p4u-card2-a-en
  !insertmacro SK_FILE c-p4u-card2-b-en
  !insertmacro SK_FILE c-p5-ck-a-en
  !insertmacro SK_FILE c-p5-ck-b-en
FunctionEnd

Function SkSetImg
  Exch $1
  Exch
  Exch $0
  Push $2
  Push $3
  System::Call 'user32::LoadImage(p 0,t r1,i 0,i $SkW,i $SkH,i 0x10)p .r2'
  SendMessage $0 0x172 0 $2 $3
  ${If} $3 <> 0
    System::Call 'gdi32::DeleteObject(p r3)'
  ${EndIf}
  Pop $3
  Pop $2
  Pop $0
  Pop $1
FunctionEnd

; Stack: control hwnd, bitmap file. Shows the bitmap scaled to the control's own size.
Function SkSetCtlImg
  Exch $1
  Exch
  Exch $0
  Push $2
  Push $3
  Push $4
  Push $5
  System::Alloc 16
  Pop $5
  System::Call 'user32::GetClientRect(p r0,p r5)'
  System::Call '*$5(i,i,i.r3,i.r4)'
  System::Free $5
  System::Call 'user32::LoadImage(p 0,t r1,i 0,i r3,i r4,i 0x10)p .r2'
  SendMessage $0 0x172 0 $2 $3
  ${If} $3 <> 0
    System::Call 'gdi32::DeleteObject(p r3)'
  ${EndIf}
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $0
  Pop $1
FunctionEnd

; Sizes the window once, hides the standard buttons and header on every page.
Function SkChrome
  Push $0
  Push $1
  Push $2
  Push $3
  Push $4
  Push $5
  Push $6
  Push $7
  ${If} $SkReady <> 1
    StrCpy $SkReady 1
    StrCpy $SkPfx "en"
    Call SkExtract
    StrCpy $0 96
    System::Call 'user32::GetDpiForWindow(p $HWNDPARENT)i .r0'
    ${If} $0 < 96
      StrCpy $0 96
    ${EndIf}
    IntOp $SkW ${SK_W} * $0
    IntOp $SkW $SkW / 96
    IntOp $SkH ${SK_H} * $0
    IntOp $SkH $SkH / 96
    System::Alloc 16
    Pop $1
    System::Call 'user32::GetWindowRect(p $HWNDPARENT,p r1)'
    System::Call '*$1(i.r2,i.r3,i.r4,i.r5)'
    IntOp $6 $4 - $2
    IntOp $7 $5 - $3
    System::Call 'user32::GetClientRect(p $HWNDPARENT,p r1)'
    System::Call '*$1(i,i,i.r4,i.r5)'
    IntOp $6 $6 - $4
    IntOp $7 $7 - $5
    IntOp $6 $6 + $SkW
    IntOp $7 $7 + $SkH
    System::Free $1
    System::Call 'user32::GetSystemMetrics(i 0)i .r2'
    System::Call 'user32::GetSystemMetrics(i 1)i .r3'
    IntOp $2 $2 - $6
    IntOp $2 $2 / 2
    IntOp $3 $3 - $7
    IntOp $3 $3 / 2
    System::Call 'user32::SetWindowPos(p $HWNDPARENT,p 0,i r2,i r3,i r6,i r7,i 0x0004)'
    GetDlgItem $1 $HWNDPARENT 1018
    System::Call 'user32::MoveWindow(p r1,i 0,i 0,i $SkW,i $SkH,i 1)'
    FindWindow $2 "#32770" "" $HWNDPARENT
    ${If} $2 <> 0
      System::Call 'user32::MoveWindow(p r2,i 0,i 0,i $SkW,i $SkH,i 1)'
    ${EndIf}
  ${EndIf}
  ; hide every outer control except the page area (NSIS shows them again on each page)
  StrCpy $1 0
  sk_hide_loop:
    FindWindow $1 "" "" $HWNDPARENT $1
    IntCmp $1 0 sk_hide_done
    System::Call 'user32::GetClassName(p r1,t .r2,i 40)'
    StrCmp $2 "#32770" sk_hide_loop
    System::Call 'user32::GetDlgCtrlID(p r1)i .r3'
    StrCmp $3 1018 sk_hide_loop
    ShowWindow $1 0
    Goto sk_hide_loop
  sk_hide_done:
  Pop $7
  Pop $6
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

Function SkNext
  SendMessage $HWNDPARENT 0x408 1 0
FunctionEnd
Function SkBack
  SendMessage $HWNDPARENT 0x408 -1 0
FunctionEnd
Function SkCancel
  SendMessage $HWNDPARENT 0x10 0 0
FunctionEnd

; Called from the install sections: Push 1|2|3, Call SkStep (checks a step and updates the percentage).
Function SkStep
  Exch $0
  Push $1
  Push $2
  Push $3
  Push $4
  Push $5
  Push $6
  Push $7
  Push $8
  Push $9
  ${If} $SkInst = 1
    SendMessage $SkProg 0x408 0 0 $1
    SendMessage $SkProg 0x407 0 0 $2
    ${If} $2 > 0
      IntOp $1 $1 * 100
      IntOp $1 $1 / $2
    ${Else}
      StrCpy $1 0
    ${EndIf}
    ${If} $0 = 3
      StrCpy $1 100
    ${EndIf}
    SendMessage $SkPct 0xC 0 "STR:$1%"
    ${If} $0 = 1
      StrCpy $2 241
    ${ElseIf} $0 = 2
      StrCpy $2 281
    ${Else}
      StrCpy $2 321
    ${EndIf}
    System::Call 'user32::CreateWindowEx(i 0,t "STATIC",t "",i 0x5000000E,i 0,i 0,i 10,i 10,p $SkDlg,p 0,p 0,p 0)p .r3'
    !insertmacro SK_PLACE $3 262 $2 22 22
    System::Call 'user32::LoadImage(p 0,t "$SkImg\chk.bmp",i 0,i r8,i r9,i 0x10)p .r4'
    SendMessage $3 0x172 0 $4
    System::Call 'gdi32::CreateEllipticRgn(i 0,i 0,i r8,i r9)p .r5'
    System::Call 'user32::SetWindowRgn(p r3,p r5,i 1)'
    System::Call 'user32::SetWindowPos(p r3,p 0,i 0,i 0,i 0,i 0,i 0x13)'
  ${EndIf}
  Pop $9
  Pop $8
  Pop $7
  Pop $6
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

; Installation page: the standard page gets the background image, the real progress bar goes on the track.
Function SkInstShow
  Push $0
  Push $1
  Push $2
  Push $3
  Push $4
  Push $5
  Push $6
  Push $7
  Push $8
  Push $9
  Call SkChrome
  FindWindow $0 "#32770" "" $HWNDPARENT
  StrCpy $SkDlg $0
  System::Call 'user32::MoveWindow(p r0,i 0,i 0,i $SkW,i $SkH,i 1)'
  GetDlgItem $1 $0 1006
  ShowWindow $1 0
  GetDlgItem $1 $0 1016
  ShowWindow $1 0
  GetDlgItem $1 $0 1027
  ShowWindow $1 0
  System::Call 'user32::CreateWindowEx(i 0,t "STATIC",t "",i 0x5000000E,i 0,i 0,i $SkW,i $SkH,p r0,p 0,p 0,p 0)p .r2'
  StrCpy $SkBg $2
  Push $2
  Push "$SkImg\p3-$SkPfx.bmp"
  Call SkSetImg
  System::Call 'user32::SetWindowPos(p r2,p 1,i 0,i 0,i 0,i 0,i 0x13)'
  GetDlgItem $3 $0 1004
  StrCpy $SkProg $3
  System::Call 'uxtheme::SetWindowTheme(p r3,w "",w "")'
  System::Call 'user32::GetWindowLong(p r3,i -16)i .r4'
  IntOp $4 $4 | 1
  System::Call 'user32::SetWindowLong(p r3,i -16,i r4)'
  SendMessage $3 0x409 0 0x00F6823B
  SendMessage $3 0x2001 0 0x0052251B
  !insertmacro SK_PLACE $3 262 188 466 10
  System::Call 'gdi32::CreateRoundRectRgn(i 0,i 0,i r8,i r9,i 10,i 10)p .r5'
  System::Call 'user32::SetWindowRgn(p r3,p r5,i 1)'
  System::Call 'user32::CreateWindowEx(i 0,t "STATIC",t "0%",i 0x50000002,i 0,i 0,i 10,i 10,p r0,p 0,p 0,p 0)p .r4'
  StrCpy $SkPct $4
  SendMessage $0 0x31 0 0 $5
  SendMessage $4 0x30 $5 1
  SetCtlColors $4 "A9B6DC" "0A112C"
  !insertmacro SK_PLACE $4 262 206 466 20
  StrCpy $SkInst 1
  Pop $9
  Pop $8
  Pop $7
  Pop $6
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

; Installer pages, must be ordered as they appear
; 1. Welcome Page
Page custom SkWelcomeShow

; 2. License Page (if defined)
!if "${LICENSE}" != ""
  !define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
  !insertmacro MUI_PAGE_LICENSE "${LICENSE}"
!endif

; 3. Install mode (if it is set to `both`)
!if "${INSTALLMODE}" == "both"
  !define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
  !insertmacro MULTIUSER_PAGE_INSTALLMODE
!endif

; 4. Custom page to ask user if he wants to reinstall/uninstall
;    only if a previous installation was detected
Var ReinstallPageCheck
Page custom PageReinstall PageLeaveReinstall

; Find a previous WiX installation and set $R6 to its uninstall registry key, or to "" if none.
;
; A WiX installer stores the installation info in registry
; using a UUID and so we have to loop through all keys under
; `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall`
; and check if `DisplayName` and `Publisher` keys match ${PRODUCTNAME} and ${MANUFACTURER}
;
; This has a potential issue that there maybe another installation that matches
; our ${PRODUCTNAME} and ${MANUFACTURER} but wasn't installed by our WiX installer,
; however, this should be fine since the user will have to confirm the uninstallation
; and they can chose to abort it if doesn't make sense.
Function FindWixInstall
  StrCpy $R6 ""
  StrCpy $0 0
  wix_loop:
    EnumRegKey $1 HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall" $0
    StrCmp $1 "" wix_loop_done ; Exit loop if there is no more keys to loop on
    IntOp $0 $0 + 1
    ReadRegStr $R0 HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\$1" "DisplayName"
    ReadRegStr $R1 HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\$1" "Publisher"
    StrCmp "$R0$R1" "${PRODUCTNAME}${MANUFACTURER}" 0 wix_loop
    ReadRegStr $R0 HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\$1" "UninstallString"
    ${StrCase} $R1 $R0 "L"
    ${StrLoc} $R0 $R1 "msiexec" ">"
    StrCmp $R0 0 0 wix_loop_done
    StrCpy $R6 "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\$1"
  wix_loop_done:
FunctionEnd

Function PageReinstall
  ; Uninstall previous WiX installation if exists.
  Call FindWixInstall
  ${If} $R6 != ""
    StrCpy $WixMode 1
    Goto compare_version
  ${EndIf}

  ; Check if there is an existing installation, if not, abort the reinstall page
  ReadRegStr $R0 SHCTX "${UNINSTKEY}" ""
  ReadRegStr $R1 SHCTX "${UNINSTKEY}" "UninstallString"
  ${IfThen} "$R0$R1" == "" ${|} Abort ${|}

  ; Compare this installar version with the existing installation
  ; and modify the messages presented to the user accordingly
  compare_version:
  StrCpy $R4 "$(older)"
  ${If} $WixMode = 1
    ReadRegStr $R0 HKLM "$R6" "DisplayVersion"
  ${Else}
    ReadRegStr $R0 SHCTX "${UNINSTKEY}" "DisplayVersion"
  ${EndIf}
  ${IfThen} $R0 == "" ${|} StrCpy $R4 "$(unknown)" ${|}

  nsis_tauri_utils::SemverCompare "${VERSION}" $R0
  Pop $R0

  ; Read the install choice header through $R5 so a custom language file that only
  ; defines the misspelled `choowHowToInstall` still shows its own text
  ; TODO: remove the `choowHowToInstall` fallback in a future major version
  StrCpy $R5 "$(chooseHowToInstall)"
  ${IfThen} $R5 == "" ${|} StrCpy $R5 "$(choowHowToInstall)" ${|}

  ; Reinstalling the same version
  ${If} $R0 = 0
    StrCpy $R1 "$(alreadyInstalledLong)"
    StrCpy $R2 "$(addOrReinstall)"
    StrCpy $R3 "$(uninstallApp)"
    !insertmacro MUI_HEADER_TEXT "$(alreadyInstalled)" "$(chooseMaintenanceOption)"
  ; Upgrading
  ${ElseIf} $R0 = 1
    StrCpy $R1 "$(olderOrUnknownVersionInstalled)"
    StrCpy $R2 "$(uninstallBeforeInstalling)"
    StrCpy $R3 "$(dontUninstall)"
    !insertmacro MUI_HEADER_TEXT "$(alreadyInstalled)" "$R5"
  ; Downgrading
  ${ElseIf} $R0 = -1
    StrCpy $R1 "$(newerVersionInstalled)"
    StrCpy $R2 "$(uninstallBeforeInstalling)"
    !if "${ALLOWDOWNGRADES}" == "true"
      StrCpy $R3 "$(dontUninstall)"
    !else
      StrCpy $R3 "$(dontUninstallDowngrade)"
    !endif
    !insertmacro MUI_HEADER_TEXT "$(alreadyInstalled)" "$R5"
  ${Else}
    Abort
  ${EndIf}

  ; Skip showing the page if passive
  ;
  ; Note that we don't call this earlier at the beginning
  ; of this function because we need to populate some variables
  ; related to current installed version if detected and whether
  ; we are downgrading or not.
  ${If} $PassiveMode = 1
    Call PageLeaveReinstall
  ${Else}
    Call SkChrome
    nsDialogs::Create 1018
    Pop $R4
    ${IfThen} $(^RTL) = 1 ${|} nsDialogs::SetRTL $(^RTL) ${|}

    ; hidden radio buttons keep the choice (PageLeaveReinstall reads $R2)
    ${NSD_CreateRadioButton} 0 0 10u 8u $R2
    Pop $R2
    ShowWindow $R2 0
    ${NSD_CreateRadioButton} 0 0 10u 8u $R3
    Pop $R3
    ShowWindow $R3 0
    ${If} $ReinstallPageCheck <> 2
      SendMessage $R2 ${BM_SETCHECK} ${BST_CHECKED} 0
    ${Else}
      SendMessage $R3 ${BM_SETCHECK} ${BST_CHECKED} 0
    ${EndIf}

    ${If} $R0 = 0
      StrCpy $SkVar "p4s"
    ${Else}
      StrCpy $SkVar "p4u"
    ${EndIf}
    !insertmacro SK_BTNX $SkC1 262 126 466 68 SkRadio1
    !insertmacro SK_BTNX $SkC2 262 206 466 68 SkRadio2
    !insertmacro SK_BTNX $0 524 411 100 39 SkCancel
    Push $0
    Push "$SkImg\c-p4s-b-cancel-$SkPfx.bmp"
    Call SkSetCtlImg
    !insertmacro SK_BTNX $0 636 411 92 39 SkNext
    Push $0
    Push "$SkImg\c-p4s-b-next-$SkPfx.bmp"
    Call SkSetCtlImg
    ${NSD_CreateBitmap} 0 0 100% 100% ""
    Pop $SkBg
    Call SkReinstallImg
    !insertmacro SK_BOTTOM
    nsDialogs::Show
  ${EndIf}
FunctionEnd
Function PageReinstallUpdateSelection
  ${NSD_GetState} $R2 $R1
  ${If} $R1 == ${BST_CHECKED}
    StrCpy $ReinstallPageCheck 1
  ${Else}
    StrCpy $ReinstallPageCheck 2
  ${EndIf}
FunctionEnd
Function PageLeaveReinstall
  ${NSD_GetState} $R2 $R1

  ; If migrating from Wix, always uninstall
  ${If} $WixMode = 1
    Goto reinst_uninstall
  ${EndIf}

  ; In update mode, always proceeds without uninstalling
  ${If} $UpdateMode = 1
    Goto reinst_done
  ${EndIf}

  ; $R0 holds whether same(0)/upgrading(1)/downgrading(-1) version
  ; $R1 holds the radio buttons state:
  ;   1 => first choice was selected
  ;   0 => second choice was selected
  ${If} $R0 = 0 ; Same version, proceed
    ${If} $R1 = 1              ; User chose to add/reinstall
      Goto reinst_done
    ${Else}                    ; User chose to uninstall
      Goto reinst_uninstall
    ${EndIf}
  ${ElseIf} $R0 = 1 ; Upgrading
    ${If} $R1 = 1              ; User chose to uninstall
      Goto reinst_uninstall
    ${Else}
      Goto reinst_done         ; User chose NOT to uninstall
    ${EndIf}
  ${ElseIf} $R0 = -1 ; Downgrading
    ${If} $R1 = 1              ; User chose to uninstall
      Goto reinst_uninstall
    ${Else}
      Goto reinst_done         ; User chose NOT to uninstall
    ${EndIf}
  ${EndIf}

  reinst_uninstall:
    HideWindow
    ClearErrors

    ${If} $WixMode = 1
      ReadRegStr $R1 HKLM "$R6" "UninstallString"
      ExecWait '$R1' $0
    ${Else}
      ReadRegStr $4 SHCTX "${MANUPRODUCTKEY}" ""
      ReadRegStr $R1 SHCTX "${UNINSTKEY}" "UninstallString"
      ${IfThen} $UpdateMode = 1 ${|} StrCpy $R1 "$R1 /UPDATE" ${|} ; append /UPDATE
      ${IfThen} $PassiveMode = 1 ${|} StrCpy $R1 "$R1 /P" ${|} ; append /P
      StrCpy $R1 "$R1 _?=$4" ; append uninstall directory
      ExecWait '$R1' $0
    ${EndIf}

    BringToFront

    ${IfThen} ${Errors} ${|} StrCpy $0 2 ${|} ; ExecWait failed, set fake exit code

    ${If} $0 <> 0
    ${OrIf} ${FileExists} "$INSTDIR\${MAINBINARYNAME}.exe"
      ; User cancelled wix uninstaller? return to select un/reinstall page
      ${If} $WixMode = 1
      ${AndIf} $0 = 1602
        Abort
      ${EndIf}

      ; User cancelled NSIS uninstaller? return to select un/reinstall page
      ${If} $0 = 1
        Abort
      ${EndIf}

      ; Other errors? show generic error message and return to select un/reinstall page
      MessageBox MB_ICONEXCLAMATION "$(unableToUninstall)"
      Abort
    ${EndIf}

    ; The NSIS uninstaller was run in place (`_?=`), so it cannot delete itself
    ; and the install directory, clean them up here once the uninstall succeeded
    ; https://nsis.sourceforge.io/Docs/Chapter3.html#:~:text=path%20contains%20spaces.-,3.2.3%20Examples,-installer.exe%20/NCRC
    ${If} $WixMode = 0
    ${AndIf} $4 != ""
      Delete "$4\uninstall.exe"
      RMDir "$4"
    ${EndIf}
  reinst_done:
FunctionEnd

; ===== skinned pages (defined after $ReinstallPageCheck) =====
Function SkReinstallImg
  ${If} $ReinstallPageCheck = 2
    StrCpy $1 b
  ${Else}
    StrCpy $1 a
  ${EndIf}
  Push $SkBg
  Push "$SkImg\$SkVar-$1-$SkPfx.bmp"
  Call SkSetImg
  Push $SkC1
  Push "$SkImg\c-$SkVar-card1-$1-$SkPfx.bmp"
  Call SkSetCtlImg
  Push $SkC2
  Push "$SkImg\c-$SkVar-card2-$1-$SkPfx.bmp"
  Call SkSetCtlImg
FunctionEnd
Function SkRadio1
  StrCpy $ReinstallPageCheck 1
  SendMessage $R2 ${BM_SETCHECK} ${BST_CHECKED} 0
  SendMessage $R3 ${BM_SETCHECK} ${BST_UNCHECKED} 0
  Call SkReinstallImg
FunctionEnd
Function SkRadio2
  !if "${ALLOWDOWNGRADES}" == "false"
    ${If} $R0 = -1
      Return
    ${EndIf}
  !endif
  StrCpy $ReinstallPageCheck 2
  SendMessage $R3 ${BM_SETCHECK} ${BST_CHECKED} 0
  SendMessage $R2 ${BM_SETCHECK} ${BST_UNCHECKED} 0
  Call SkReinstallImg
FunctionEnd

Function SkWelcomeShow
  ${If} $PassiveMode = 1
    Abort
  ${EndIf}
  Call SkChrome
  nsDialogs::Create 1018
  Pop $0
  ${If} $0 == error
    Abort
  ${EndIf}
  !insertmacro SK_BTN $1 524 411 100 39 SkCancel c-p1-b-cancel
  !insertmacro SK_BTN $1 636 411 92 39 SkNext c-p1-b-next
  !insertmacro SK_BG p1
  !insertmacro SK_BOTTOM
  nsDialogs::Show
FunctionEnd

Function SkBrowse
  ${NSD_GetText} $SkDir $1
  nsDialogs::SelectFolderDialog "" "$1"
  Pop $0
  ${If} $0 != error
    StrLen $2 "${PRODUCTNAME}"
    StrCpy $3 $0 "" -$2
    ${If} $3 != "${PRODUCTNAME}"
      StrCpy $4 $0 1 -1
      ${If} $4 == "\"
        StrCpy $0 "$0${PRODUCTNAME}"
      ${Else}
        StrCpy $0 "$0\${PRODUCTNAME}"
      ${EndIf}
    ${EndIf}
    ${NSD_SetText} $SkDir $0
  ${EndIf}
FunctionEnd

Function SkDirShow
  ${If} $PassiveMode = 1
    Abort
  ${EndIf}
  Call SkChrome
  ; space required (MB) and available (GB)
  IntOp $2 ${ESTIMATEDSIZE} / 1024
  ${If} $2 < 1
    StrCpy $2 1
  ${EndIf}
  StrCpy $2 "$2 MB"
  StrCpy $3 $INSTDIR 3
  StrCpy $4 ""
  System::Call 'kernel32::GetDiskFreeSpaceExW(w r3,*l .r5,*l,*l)i .r1'
  ${If} $1 <> 0
    System::Int64Op $5 / 1073741824
    Pop $4
    StrCpy $4 "$4 GB"
  ${EndIf}
  nsDialogs::Create 1018
  Pop $0
  ${If} $0 == error
    Abort
  ${EndIf}
  ; path field (borderless, same colours as the card)
  ${NSD_CreateText} 0 0 10u 10u "$INSTDIR"
  Pop $SkDir
  System::Call 'user32::GetWindowLong(p $SkDir,i -20)i .r1'
  IntOp $1 $1 ^ 0x200
  System::Call 'user32::SetWindowLong(p $SkDir,i -20,i r1)'
  System::Call 'user32::SetWindowPos(p $SkDir,p 0,i 0,i 0,i 0,i 0,i 0x27)'
  SetCtlColors $SkDir "EAF0FF" "0F1740"
  !insertmacro SK_PLACE $SkDir 262 184 346 44
  IntOp $7 $7 + 12
  IntOp $9 $9 - 24
  IntOp $6 $6 + 14
  IntOp $8 $8 - 28
  System::Call 'user32::MoveWindow(p $SkDir,i r6,i r7,i r8,i r9,i 1)'
  CreateFont $R9 "Segoe UI Semibold" 11 600
  ${NSD_CreateLabel} 0 0 10u 10u "$2"
  Pop $1
  SetCtlColors $1 "EAF0FF" "0F1740"
  SendMessage $1 0x30 $R9 1
  !insertmacro SK_PLACE $1 279 279 193 30
  ${NSD_CreateLabel} 0 0 10u 10u "$4"
  Pop $1
  SetCtlColors $1 "EAF0FF" "0F1740"
  SendMessage $1 0x30 $R9 1
  !insertmacro SK_PLACE $1 518 279 193 30
  !insertmacro SK_BTN $1 618 184 110 44 SkBrowse c-p2-b-browse
  !insertmacro SK_BTN $1 516 411 100 39 SkBack c-p2-b-back
  !insertmacro SK_BTN $1 628 411 100 39 SkNext c-p2-b-install
  !insertmacro SK_BG p2
  !insertmacro SK_BOTTOM
  nsDialogs::Show
FunctionEnd
Function SkDirLeave
  ${NSD_GetText} $SkDir $0
  ${If} $0 == ""
    Abort
  ${EndIf}
  StrCpy $INSTDIR $0
FunctionEnd

Function SkToggleRun
  ${If} $SkRun = 1
    StrCpy $SkRun 0
    StrCpy $1 b
  ${Else}
    StrCpy $SkRun 1
    StrCpy $1 a
  ${EndIf}
  Push $SkBg
  Push "$SkImg\p5-$1-$SkPfx.bmp"
  Call SkSetImg
  Push $SkCk
  Push "$SkImg\c-p5-ck-$1-$SkPfx.bmp"
  Call SkSetCtlImg
FunctionEnd
Function SkFinishShow
  ${If} $PassiveMode = 1
    Abort
  ${EndIf}
  Call SkChrome
  nsDialogs::Create 1018
  Pop $0
  ${If} $0 == error
    Abort
  ${EndIf}
  StrCpy $SkRun 1
  !insertmacro SK_BTN $SkCk 262 134 466 56 SkToggleRun c-p5-ck-a
  !insertmacro SK_BTN $1 628 411 100 39 SkNext c-p5-b-fin
  !insertmacro SK_BG p5-a
  !insertmacro SK_BOTTOM
  nsDialogs::Show
FunctionEnd
Function SkFinishLeave
  ${If} $SkRun = 1
    Call RunMainBinary
  ${EndIf}
FunctionEnd

; 5. Choose install directory page
Page custom SkDirShow SkDirLeave

; 6. Start menu shortcut page
Var AppStartMenuFolder
!define MUI_PAGE_CUSTOMFUNCTION_PRE Skip
!insertmacro MUI_PAGE_STARTMENU Application $AppStartMenuFolder

; 7. Installation page
!define MUI_PAGE_CUSTOMFUNCTION_SHOW SkInstShow
!insertmacro MUI_PAGE_INSTFILES

; 8. Finish page
;
Page custom SkFinishShow SkFinishLeave

Function RunMainBinary
  nsis_tauri_utils::RunAsUser "$INSTDIR\${MAINBINARYNAME}.exe" ""
FunctionEnd

; Uninstaller Pages
; 1. Confirm uninstall page
Var DeleteAppDataCheckbox
Var DeleteAppDataCheckboxState
!define /ifndef WS_EX_LAYOUTRTL         0x00400000
; ===== skinned uninstaller =====
!macro UNSK_BTNX VAR X Y W H FUNC
  nsDialogs::CreateControl STATIC 0x5400010E 0 0 0 10u 10u ""
  Pop ${VAR}
  !insertmacro SK_PLACE ${VAR} ${X} ${Y} ${W} ${H}
  ${NSD_OnClick} ${VAR} ${FUNC}
!macroend
Function un.SkSetImg
  Exch $1
  Exch
  Exch $0
  Push $2
  Push $3
  System::Call 'user32::LoadImage(p 0,t r1,i 0,i $SkW,i $SkH,i 0x10)p .r2'
  SendMessage $0 0x172 0 $2 $3
  ${If} $3 <> 0
    System::Call 'gdi32::DeleteObject(p r3)'
  ${EndIf}
  Pop $3
  Pop $2
  Pop $0
  Pop $1
FunctionEnd

; Stack: control hwnd, bitmap file. Shows the bitmap scaled to the control's own size.
Function un.SkSetCtlImg
  Exch $1
  Exch
  Exch $0
  Push $2
  Push $3
  Push $4
  Push $5
  System::Alloc 16
  Pop $5
  System::Call 'user32::GetClientRect(p r0,p r5)'
  System::Call '*$5(i,i,i.r3,i.r4)'
  System::Free $5
  System::Call 'user32::LoadImage(p 0,t r1,i 0,i r3,i r4,i 0x10)p .r2'
  SendMessage $0 0x172 0 $2 $3
  ${If} $3 <> 0
    System::Call 'gdi32::DeleteObject(p r3)'
  ${EndIf}
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $0
  Pop $1
FunctionEnd

; Sizes the window once, hides the standard buttons and header on every page.
Function un.SkChrome
  Push $0
  Push $1
  Push $2
  Push $3
  Push $4
  Push $5
  Push $6
  Push $7
  ${If} $SkReady <> 1
    StrCpy $SkReady 1
    StrCpy $SkPfx "en"
    StrCpy $SkImg "$INSTDIR\skin"
    StrCpy $0 96
    System::Call 'user32::GetDpiForWindow(p $HWNDPARENT)i .r0'
    ${If} $0 < 96
      StrCpy $0 96
    ${EndIf}
    IntOp $SkW ${SK_W} * $0
    IntOp $SkW $SkW / 96
    IntOp $SkH ${SK_H} * $0
    IntOp $SkH $SkH / 96
    System::Alloc 16
    Pop $1
    System::Call 'user32::GetWindowRect(p $HWNDPARENT,p r1)'
    System::Call '*$1(i.r2,i.r3,i.r4,i.r5)'
    IntOp $6 $4 - $2
    IntOp $7 $5 - $3
    System::Call 'user32::GetClientRect(p $HWNDPARENT,p r1)'
    System::Call '*$1(i,i,i.r4,i.r5)'
    IntOp $6 $6 - $4
    IntOp $7 $7 - $5
    IntOp $6 $6 + $SkW
    IntOp $7 $7 + $SkH
    System::Free $1
    System::Call 'user32::GetSystemMetrics(i 0)i .r2'
    System::Call 'user32::GetSystemMetrics(i 1)i .r3'
    IntOp $2 $2 - $6
    IntOp $2 $2 / 2
    IntOp $3 $3 - $7
    IntOp $3 $3 / 2
    System::Call 'user32::SetWindowPos(p $HWNDPARENT,p 0,i r2,i r3,i r6,i r7,i 0x0004)'
    GetDlgItem $1 $HWNDPARENT 1018
    System::Call 'user32::MoveWindow(p r1,i 0,i 0,i $SkW,i $SkH,i 1)'
    FindWindow $2 "#32770" "" $HWNDPARENT
    ${If} $2 <> 0
      System::Call 'user32::MoveWindow(p r2,i 0,i 0,i $SkW,i $SkH,i 1)'
    ${EndIf}
  ${EndIf}
  ; hide every outer control except the page area (NSIS shows them again on each page)
  StrCpy $1 0
  sk_hide_loop:
    FindWindow $1 "" "" $HWNDPARENT $1
    IntCmp $1 0 sk_hide_done
    System::Call 'user32::GetClassName(p r1,t .r2,i 40)'
    StrCmp $2 "#32770" sk_hide_loop
    System::Call 'user32::GetDlgCtrlID(p r1)i .r3'
    StrCmp $3 1018 sk_hide_loop
    ShowWindow $1 0
    Goto sk_hide_loop
  sk_hide_done:
  Pop $7
  Pop $6
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

Function un.SkNext
  SendMessage $HWNDPARENT 0x408 1 0
FunctionEnd
Function un.SkBack
  SendMessage $HWNDPARENT 0x408 -1 0
FunctionEnd
Function un.SkCancel
  SendMessage $HWNDPARENT 0x10 0 0
FunctionEnd

Function un.SkToggleData
  ${If} $DeleteAppDataCheckboxState = 1
    StrCpy $DeleteAppDataCheckboxState 0
    StrCpy $1 a
  ${Else}
    StrCpy $DeleteAppDataCheckboxState 1
    StrCpy $1 b
  ${EndIf}
  Push $SkBg
  Push "$SkImg\u1-$1-$SkPfx.bmp"
  Call un.SkSetImg
  Push $SkCk
  Push "$SkImg\c-u1-ck-$1-$SkPfx.bmp"
  Call un.SkSetCtlImg
FunctionEnd

Function un.SkConfirmShow
  ${If} $PassiveMode = 1
    Abort
  ${EndIf}
  Call un.SkChrome
  nsDialogs::Create 1018
  Pop $0
  ${If} $0 == error
    Abort
  ${EndIf}
  StrCpy $DeleteAppDataCheckboxState 0
  ${NSD_CreateLabel} 0 0 10u 10u "$INSTDIR"
  Pop $1
  SetCtlColors $1 "EAF0FF" "0F1740"
  !insertmacro SK_PLACE $1 279 167 432 30
  !insertmacro UNSK_BTNX $SkCk 262 224 466 56 un.SkToggleData
  Push $SkCk
  Push "$SkImg\c-u1-ck-a-$SkPfx.bmp"
  Call un.SkSetCtlImg
  !insertmacro UNSK_BTNX $1 496 411 100 39 un.SkCancel
  Push $1
  Push "$SkImg\c-u1-b-cancel-$SkPfx.bmp"
  Call un.SkSetCtlImg
  !insertmacro UNSK_BTNX $1 608 411 120 39 un.SkNext
  Push $1
  Push "$SkImg\c-u1-b-uninst-$SkPfx.bmp"
  Call un.SkSetCtlImg
  ${NSD_CreateBitmap} 0 0 100% 100% ""
  Pop $SkBg
  Push $SkBg
  Push "$SkImg\u1-a-$SkPfx.bmp"
  Call un.SkSetImg
  System::Call 'user32::SetWindowPos(p $SkBg,p 1,i 0,i 0,i 0,i 0,i 0x13)'
  nsDialogs::Show
FunctionEnd

Function un.SkInstShow
  Push $0
  Push $1
  Push $2
  Push $3
  Push $4
  Push $5
  Push $6
  Push $7
  Push $8
  Push $9
  Call un.SkChrome
  FindWindow $0 "#32770" "" $HWNDPARENT
  System::Call 'user32::MoveWindow(p r0,i 0,i 0,i $SkW,i $SkH,i 1)'
  GetDlgItem $1 $0 1006
  ShowWindow $1 0
  GetDlgItem $1 $0 1016
  ShowWindow $1 0
  GetDlgItem $1 $0 1027
  ShowWindow $1 0
  System::Call 'user32::CreateWindowEx(i 0,t "STATIC",t "",i 0x5000000E,i 0,i 0,i $SkW,i $SkH,p r0,p 0,p 0,p 0)p .r2'
  Push $2
  Push "$SkImg\u2-$SkPfx.bmp"
  Call un.SkSetImg
  System::Call 'user32::SetWindowPos(p r2,p 1,i 0,i 0,i 0,i 0,i 0x13)'
  GetDlgItem $3 $0 1004
  System::Call 'uxtheme::SetWindowTheme(p r3,w "",w "")'
  System::Call 'user32::GetWindowLong(p r3,i -16)i .r4'
  IntOp $4 $4 | 1
  System::Call 'user32::SetWindowLong(p r3,i -16,i r4)'
  SendMessage $3 0x409 0 0x00F6823B
  SendMessage $3 0x2001 0 0x0052251B
  !insertmacro SK_PLACE $3 262 136 466 10
  System::Call 'gdi32::CreateRoundRectRgn(i 0,i 0,i r8,i r9,i 10,i 10)p .r5'
  System::Call 'user32::SetWindowRgn(p r3,p r5,i 1)'
  Pop $9
  Pop $8
  Pop $7
  Pop $6
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

UninstPage custom un.SkConfirmShow


; 2. Uninstalling Page
!define MUI_PAGE_CUSTOMFUNCTION_SHOW un.SkInstShow
!insertmacro MUI_UNPAGE_INSTFILES

;Languages
; PageReinstall reads the misspelled `choowHowToInstall` as a fallback, and makensis warns
; about a language string that is read but never set, so default it to an empty string for
; every language. This runs before the language files are included, so a custom language
; file that only defines the old spelling overrides the default and the fallback shows its
; text. `0` as the language id means the language inserted right above.
; TODO: remove the `choowHowToInstall` fallback in a future major version
{{#each languages}}
!insertmacro MUI_LANGUAGE "{{this}}"
LangString choowHowToInstall 0 ""
{{/each}}
!insertmacro MUI_RESERVEFILE_LANGDLL
{{#each language_files}}
  !include "{{this}}"
{{/each}}

Function .onInit
  ${GetOptions} $CMDLINE "/P" $PassiveMode
  ${IfNot} ${Errors}
    StrCpy $PassiveMode 1
  ${EndIf}

  ${GetOptions} $CMDLINE "/NS" $NoShortcutMode
  ${IfNot} ${Errors}
    StrCpy $NoShortcutMode 1
  ${EndIf}

  ${GetOptions} $CMDLINE "/UPDATE" $UpdateMode
  ${IfNot} ${Errors}
    StrCpy $UpdateMode 1
  ${EndIf}

  !if "${DISPLAYLANGUAGESELECTOR}" == "true"
    !insertmacro MUI_LANGDLL_DISPLAY
  !endif

  !insertmacro SetContext

  ${If} $INSTDIR == "${PLACEHOLDER_INSTALL_DIR}"
    ; Set default install location
    !if "${INSTALLMODE}" == "perMachine"
      ${If} ${RunningX64}
        !if "${ARCH}" == "x64"
          StrCpy $INSTDIR "$PROGRAMFILES64\${PRODUCTNAME}"
        !else if "${ARCH}" == "arm64"
          StrCpy $INSTDIR "$PROGRAMFILES64\${PRODUCTNAME}"
        !else
          StrCpy $INSTDIR "$PROGRAMFILES\${PRODUCTNAME}"
        !endif
      ${Else}
        StrCpy $INSTDIR "$PROGRAMFILES\${PRODUCTNAME}"
      ${EndIf}
    !else if "${INSTALLMODE}" == "currentUser"
      StrCpy $INSTDIR "$LOCALAPPDATA\${PRODUCTNAME}"
    !endif

    Call RestorePreviousInstallLocation
  ${EndIf}


  !if "${INSTALLMODE}" == "both"
    !insertmacro MULTIUSER_INIT
  !endif

  ; Abort downgrades if they are disabled, in the modes that skip the reinstall page choice:
  ; silent, passive and update
  !if "${ALLOWDOWNGRADES}" == "false"
  ${If} ${Silent}
  ${OrIf} $PassiveMode = 1
  ${OrIf} $UpdateMode = 1
    ; Compare with the same installation the reinstall page would: a WiX one first
    Call FindWixInstall
    ${If} $R6 != ""
      ReadRegStr $R0 HKLM "$R6" "DisplayVersion"
    ${Else}
      ReadRegStr $R0 SHCTX "${UNINSTKEY}" "DisplayVersion"
    ${EndIf}
    nsis_tauri_utils::SemverCompare "${VERSION}" $R0
    Pop $R0
    ; If downgrading
    ${If} $R0 = -1
      ${If} ${Silent}
        System::Call 'kernel32::AttachConsole(i -1)i.r0'
        ${If} $0 <> 0
          System::Call 'kernel32::GetStdHandle(i -11)i.r0'
          System::call 'kernel32::SetConsoleTextAttribute(i r0, i 0x0004)' ; set red color
          FileWrite $0 "$(silentDowngrades)"
        ${EndIf}
      ${Else}
        MessageBox MB_ICONSTOP "$(silentDowngrades)"
      ${EndIf}
      Abort
    ${EndIf}
  ${EndIf}
  !endif
FunctionEnd

Section WebView2
  ; Check if Webview2 is already installed and skip this section
  ${If} ${RunningX64}
    ReadRegStr $4 HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\${WEBVIEW2APPGUID}" "pv"
  ${Else}
    ReadRegStr $4 HKLM "SOFTWARE\Microsoft\EdgeUpdate\Clients\${WEBVIEW2APPGUID}" "pv"
  ${EndIf}
  ${If} $4 == ""
    ReadRegStr $4 HKCU "SOFTWARE\Microsoft\EdgeUpdate\Clients\${WEBVIEW2APPGUID}" "pv"
  ${EndIf}

  ${If} $4 == ""
    ; Webview2 installation
    ;
    ; Skip if updating
    ${If} $UpdateMode <> 1
      !if "${INSTALLWEBVIEW2MODE}" == "downloadBootstrapper"
        Delete "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        DetailPrint "$(webview2Downloading)"
        NSISdl::download "https://go.microsoft.com/fwlink/p/?LinkId=2124703" "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        Pop $0
        ${If} $0 == "success"
          DetailPrint "$(webview2DownloadSuccess)"
        ${Else}
          DetailPrint "$(webview2DownloadError)"
          Abort "$(webview2AbortError)"
        ${EndIf}
        StrCpy $6 "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        Goto install_webview2
      !endif

      !if "${INSTALLWEBVIEW2MODE}" == "embedBootstrapper"
        Delete "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        File "/oname=$TEMP\MicrosoftEdgeWebview2Setup.exe" "${WEBVIEW2BOOTSTRAPPERPATH}"
        DetailPrint "$(installingWebview2)"
        StrCpy $6 "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        Goto install_webview2
      !endif

      !if "${INSTALLWEBVIEW2MODE}" == "offlineInstaller"
        Delete "$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe"
        File "/oname=$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe" "${WEBVIEW2INSTALLERPATH}"
        DetailPrint "$(installingWebview2)"
        StrCpy $6 "$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe"
        Goto install_webview2
      !endif

      Goto webview2_done

      install_webview2:
        DetailPrint "$(installingWebview2)"
        ; $6 holds the path to the webview2 installer
        ExecWait "$6 ${WEBVIEW2INSTALLERARGS} /install" $1
        ${If} $1 = 0
          DetailPrint "$(webview2InstallSuccess)"
        ${Else}
          DetailPrint "$(webview2InstallError)"
          Abort "$(webview2AbortError)"
        ${EndIf}
      webview2_done:
    ${EndIf}
  ${Else}
    !if "${MINIMUMWEBVIEW2VERSION}" != ""
      ${VersionCompare} "${MINIMUMWEBVIEW2VERSION}" "$4" $R0
      ${If} $R0 = 1
        update_webview:
          DetailPrint "$(installingWebview2)"
          ${If} ${RunningX64}
            ReadRegStr $R1 HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate" "path"
          ${Else}
            ReadRegStr $R1 HKLM "SOFTWARE\Microsoft\EdgeUpdate" "path"
          ${EndIf}
          ${If} $R1 == ""
            ReadRegStr $R1 HKCU "SOFTWARE\Microsoft\EdgeUpdate" "path"
          ${EndIf}
          ${If} $R1 != ""
            ; Chromium updater docs: https://source.chromium.org/chromium/chromium/src/+/main:docs/updater/user_manual.md
            ; Modified from "HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Microsoft EdgeWebView\ModifyPath"
            ExecWait `"$R1" /install appguid=${WEBVIEW2APPGUID}&needsadmin=true` $1
            ${If} $1 = 0
              DetailPrint "$(webview2InstallSuccess)"
            ${Else}
              MessageBox MB_ICONEXCLAMATION|MB_ABORTRETRYIGNORE "$(webview2InstallError)" IDIGNORE ignore IDRETRY update_webview
              Quit
              ignore:
            ${EndIf}
          ${EndIf}
      ${EndIf}
    !endif
  ${EndIf}
SectionEnd

Section Install
  Push 1
  Call SkStep
  SetOutPath $INSTDIR

  !ifmacrodef NSIS_HOOK_PREINSTALL
    !insertmacro NSIS_HOOK_PREINSTALL
  !endif

  !insertmacro CheckIfAppIsRunning "$INSTDIR\${MAINBINARYNAME}.exe" "${PRODUCTNAME}"

  ; Copy main executable
  File "${MAINBINARYSRCPATH}"

  ; Copy resources
  {{#each resources_dirs}}
    CreateDirectory "$INSTDIR\\{{this}}"
  {{/each}}
  {{#each resources}}
    File /a "/oname={{this.[1]}}" "{{no-escape @key}}"
  {{/each}}

  ; Copy external binaries
  {{#each binaries}}
    File /a "/oname={{this}}" "{{no-escape @key}}"
  {{/each}}

  ; Create file associations
  {{#each file_associations as |association| ~}}
    {{#each association.ext as |ext| ~}}
       !insertmacro APP_ASSOCIATE "{{ext}}" "{{or association.name ext}}" "{{association-description association.description ext}}" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\",0" "Open with ${PRODUCTNAME}" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\" $\"%1$\""
    {{/each}}
  {{/each}}

  ; Register deep links
  {{#each deep_link_protocols as |protocol| ~}}
    WriteRegStr SHCTX "Software\Classes\\{{protocol}}" "URL Protocol" ""
    WriteRegStr SHCTX "Software\Classes\\{{protocol}}" "" "URL:${BUNDLEID} protocol"
    WriteRegStr SHCTX "Software\Classes\\{{protocol}}\DefaultIcon" "" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\",0"
    WriteRegStr SHCTX "Software\Classes\\{{protocol}}\shell\open\command" "" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\" $\"%1$\""
  {{/each}}

  Push 2
  Call SkStep

  SetOutPath "$INSTDIR\skin"
  File "${SK_DIR}u1-a-en.bmp"
  File "${SK_DIR}u1-b-en.bmp"
  File "${SK_DIR}u2-en.bmp"
  File "${SK_DIR}c-u1-ck-a-en.bmp"
  File "${SK_DIR}c-u1-ck-b-en.bmp"
  File "${SK_DIR}c-u1-b-cancel-en.bmp"
  File "${SK_DIR}c-u1-b-uninst-en.bmp"
  File "${SK_DIR}chk.bmp"
  SetOutPath $INSTDIR

  ; Create uninstaller
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Save $INSTDIR in registry for future installations
  WriteRegStr SHCTX "${MANUPRODUCTKEY}" "" $INSTDIR

  !if "${INSTALLMODE}" == "both"
    ; Save install mode to be selected by default for the next installation such as updating
    ; or when uninstalling
    WriteRegStr SHCTX "${UNINSTKEY}" $MultiUser.InstallMode 1
  !endif

  ; Remove old main binary if it doesn't match new main binary name
  ReadRegStr $OldMainBinaryName SHCTX "${UNINSTKEY}" "MainBinaryName"
  ${If} $OldMainBinaryName != ""
  ${AndIf} $OldMainBinaryName != "${MAINBINARYNAME}.exe"
    Delete "$INSTDIR\$OldMainBinaryName"
  ${EndIf}

  ; Save current MAINBINARYNAME for future updates
  WriteRegStr SHCTX "${UNINSTKEY}" "MainBinaryName" "${MAINBINARYNAME}.exe"

  ; Registry information for add/remove programs
  WriteRegStr SHCTX "${UNINSTKEY}" "DisplayName" "${PRODUCTNAME}"
  WriteRegStr SHCTX "${UNINSTKEY}" "DisplayIcon" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\""
  WriteRegStr SHCTX "${UNINSTKEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr SHCTX "${UNINSTKEY}" "Publisher" "${MANUFACTURER}"
  WriteRegStr SHCTX "${UNINSTKEY}" "InstallLocation" "$\"$INSTDIR$\""
  WriteRegStr SHCTX "${UNINSTKEY}" "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
  WriteRegDWORD SHCTX "${UNINSTKEY}" "NoModify" "1"
  WriteRegDWORD SHCTX "${UNINSTKEY}" "NoRepair" "1"

  ${GetSize} "$INSTDIR" "/M=uninstall.exe /S=0K /G=0" $0 $1 $2
  IntOp $0 $0 + ${ESTIMATEDSIZE}
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD SHCTX "${UNINSTKEY}" "EstimatedSize" "$0"

  !if "${HOMEPAGE}" != ""
    WriteRegStr SHCTX "${UNINSTKEY}" "URLInfoAbout" "${HOMEPAGE}"
    WriteRegStr SHCTX "${UNINSTKEY}" "URLUpdateInfo" "${HOMEPAGE}"
    WriteRegStr SHCTX "${UNINSTKEY}" "HelpLink" "${HOMEPAGE}"
  !endif

  ; Create start menu shortcut
  !insertmacro MUI_STARTMENU_WRITE_BEGIN Application
    Call CreateOrUpdateStartMenuShortcut
  !insertmacro MUI_STARTMENU_WRITE_END

  ; Create desktop shortcut for silent and passive installers
  ; because finish page will be skipped
  ${If} $PassiveMode = 1
  ${OrIf} ${Silent}
    Call CreateOrUpdateDesktopShortcut
  ${EndIf}

  !ifmacrodef NSIS_HOOK_POSTINSTALL
    !insertmacro NSIS_HOOK_POSTINSTALL
  !endif

  Push 3
  Call SkStep

  ; Move on to the finish page by itself (and close at once in passive mode)
  SetAutoClose true
SectionEnd

Function .onInstSuccess
  ; Check for `/R` flag only in silent and passive installers because
  ; GUI installer has a toggle for the user to (re)start the app
  ${If} $PassiveMode = 1
  ${OrIf} ${Silent}
    ${GetOptions} $CMDLINE "/R" $R0
    ${IfNot} ${Errors}
      ${GetOptions} $CMDLINE "/ARGS" $R0
      nsis_tauri_utils::RunAsUser "$INSTDIR\${MAINBINARYNAME}.exe" "$R0"
    ${EndIf}
  ${EndIf}
FunctionEnd

Function un.onInit
  !insertmacro SetContext

  !if "${INSTALLMODE}" == "both"
    !insertmacro MULTIUSER_UNINIT
  !endif

  !insertmacro MUI_UNGETLANGUAGE

  ${GetOptions} $CMDLINE "/P" $PassiveMode
  ${IfNot} ${Errors}
    StrCpy $PassiveMode 1
  ${EndIf}

  ${GetOptions} $CMDLINE "/UPDATE" $UpdateMode
  ${IfNot} ${Errors}
    StrCpy $UpdateMode 1
  ${EndIf}
FunctionEnd

Section Uninstall
  SetAutoClose true
  RMDir /r "$INSTDIR\skin"

  !ifmacrodef NSIS_HOOK_PREUNINSTALL
    !insertmacro NSIS_HOOK_PREUNINSTALL
  !endif

  !insertmacro CheckIfAppIsRunning "$INSTDIR\${MAINBINARYNAME}.exe" "${PRODUCTNAME}"

  ; Delete the app directory and its content from disk
  ; Copy main executable
  Delete "$INSTDIR\${MAINBINARYNAME}.exe"

  ; Delete resources
  {{#each resources}}
    Delete "$INSTDIR\\{{this.[1]}}"
  {{/each}}

  ; Delete external binaries
  {{#each binaries}}
    Delete "$INSTDIR\\{{this}}"
  {{/each}}

  ; Delete app associations
  {{#each file_associations as |association| ~}}
    {{#each association.ext as |ext| ~}}
      !insertmacro APP_UNASSOCIATE "{{ext}}" "{{or association.name ext}}"
    {{/each}}
  {{/each}}

  ; Delete deep links
  {{#each deep_link_protocols as |protocol| ~}}
    ReadRegStr $R7 SHCTX "Software\Classes\\{{protocol}}\shell\open\command" ""
    ${If} $R7 == "$\"$INSTDIR\${MAINBINARYNAME}.exe$\" $\"%1$\""
      DeleteRegKey SHCTX "Software\Classes\\{{protocol}}"
    ${EndIf}
  {{/each}}


  ; Delete uninstaller
  Delete "$INSTDIR\uninstall.exe"

  {{#each resources_ancestors}}
  RMDir /REBOOTOK "$INSTDIR\\{{this}}"
  {{/each}}
  RMDir "$INSTDIR"

  ; Remove shortcuts if not updating
  ${If} $UpdateMode <> 1
    !insertmacro DeleteAppUserModelId

    ; Remove start menu shortcut
    !insertmacro MUI_STARTMENU_GETFOLDER Application $AppStartMenuFolder
    !insertmacro IsShortcutTarget "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    Pop $0
    ${If} $0 = 1
      !insertmacro UnpinShortcut "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
      Delete "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
      RMDir "$SMPROGRAMS\$AppStartMenuFolder"
    ${EndIf}
    !insertmacro IsShortcutTarget "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    Pop $0
    ${If} $0 = 1
      !insertmacro UnpinShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk"
      Delete "$SMPROGRAMS\${PRODUCTNAME}.lnk"
    ${EndIf}

    ; Remove desktop shortcuts
    !insertmacro IsShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    Pop $0
    ${If} $0 = 1
      !insertmacro UnpinShortcut "$DESKTOP\${PRODUCTNAME}.lnk"
      Delete "$DESKTOP\${PRODUCTNAME}.lnk"
    ${EndIf}
  ${EndIf}

  ; Remove registry information for add/remove programs
  !if "${INSTALLMODE}" == "both"
    DeleteRegKey SHCTX "${UNINSTKEY}"
  !else if "${INSTALLMODE}" == "perMachine"
    DeleteRegKey HKLM "${UNINSTKEY}"
  !else
    DeleteRegKey HKCU "${UNINSTKEY}"
  !endif

  ; Removes the Autostart entry for ${PRODUCTNAME} from the HKCU Run key if it exists.
  ; This ensures the program does not launch automatically after uninstallation if it exists.
  ; If it doesn't exist, it does nothing.
  ; We do this when not updating (to preserve the registry value on updates)
  ${If} $UpdateMode <> 1
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}"
  ${EndIf}

  ; Delete app data if the checkbox is selected
  ; and if not updating
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    ; Clear the install location $INSTDIR from registry
    DeleteRegKey SHCTX "${MANUPRODUCTKEY}"
    DeleteRegKey /ifempty SHCTX "${MANUKEY}"

    ; Clear the install language from registry
    DeleteRegValue HKCU "${MANUPRODUCTKEY}" "Installer Language"
    DeleteRegKey /ifempty HKCU "${MANUPRODUCTKEY}"
    DeleteRegKey /ifempty HKCU "${MANUKEY}"

    SetShellVarContext current
    RmDir /r "$APPDATA\${BUNDLEID}"
    RmDir /r "$LOCALAPPDATA\${BUNDLEID}"
  ${EndIf}

  !ifmacrodef NSIS_HOOK_POSTUNINSTALL
    !insertmacro NSIS_HOOK_POSTUNINSTALL
  !endif

  ; Auto close if passive mode or updating
  ${If} $PassiveMode = 1
  ${OrIf} $UpdateMode = 1
    SetAutoClose true
  ${EndIf}
SectionEnd

Function RestorePreviousInstallLocation
  ReadRegStr $4 SHCTX "${MANUPRODUCTKEY}" ""
  StrCmp $4 "" +2 0
    StrCpy $INSTDIR $4
FunctionEnd

Function Skip
  Abort
FunctionEnd

Function SkipIfPassive
  ${IfThen} $PassiveMode = 1  ${|} Abort ${|}
FunctionEnd
Function un.SkipIfPassive
  ${IfThen} $PassiveMode = 1  ${|} Abort ${|}
FunctionEnd

Function CreateOrUpdateStartMenuShortcut
  ; We used to use product name as MAINBINARYNAME
  ; migrate old shortcuts to target the new MAINBINARYNAME
  StrCpy $R0 0

  !insertmacro IsShortcutTarget "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$INSTDIR\$OldMainBinaryName"
  Pop $0
  ${If} $0 = 1
    !insertmacro SetShortcutTarget "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    StrCpy $R0 1
  ${EndIf}

  !insertmacro IsShortcutTarget "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\$OldMainBinaryName"
  Pop $0
  ${If} $0 = 1
    !insertmacro SetShortcutTarget "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    StrCpy $R0 1
  ${EndIf}

  ${If} $R0 = 1
    Return
  ${EndIf}

  ; Skip creating shortcut if in update mode or no shortcut mode
  ; but always create if migrating from wix
  ${If} $WixMode = 0
    ${If} $UpdateMode = 1
    ${OrIf} $NoShortcutMode = 1
      Return
    ${EndIf}
  ${EndIf}

  !if "${STARTMENUFOLDER}" != ""
    CreateDirectory "$SMPROGRAMS\$AppStartMenuFolder"
    CreateShortcut "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
  !else
    CreateShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\${PRODUCTNAME}.lnk"
  !endif
FunctionEnd

Function CreateOrUpdateDesktopShortcut
  ; We used to use product name as MAINBINARYNAME
  ; migrate old shortcuts to target the new MAINBINARYNAME
  !insertmacro IsShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\$OldMainBinaryName"
  Pop $0
  ${If} $0 = 1
    !insertmacro SetShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    Return
  ${EndIf}

  ; Skip creating shortcut if in update mode or no shortcut mode
  ; but always create if migrating from wix
  ${If} $WixMode = 0
    ${If} $UpdateMode = 1
    ${OrIf} $NoShortcutMode = 1
      Return
    ${EndIf}
  ${EndIf}

  CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
  !insertmacro SetLnkAppUserModelId "$DESKTOP\${PRODUCTNAME}.lnk"
FunctionEnd
