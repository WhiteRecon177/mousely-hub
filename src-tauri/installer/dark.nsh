; ===== Mousely Hub dark installer: inserted into Tauri's NSIS template by make-dark-installer.ps1 =====
!include LogicLib.nsh

!define DK_TEXT "ECECF0"
!define DK_BG "0F0F12"
!define DK_BG2 "1B1B20"
; DWM wants COLORREF = 0x00BBGGRR
!define DK_BGREF 0x00120F0F
!define DK_TXTREF 0x00F0ECEC

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
    SendMessage $1 0x409 0 0x00F0ECEC
    SendMessage $1 0x2001 0 0x00302A2A
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
FunctionEnd
!macroend
!insertmacro DK_DEFINE ""
!insertmacro DK_DEFINE "un."
; ===== end of dark installer block =====

