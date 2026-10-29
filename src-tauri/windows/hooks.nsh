; Aggiunte all'installer NSIS di Tauri: le DLL che l'eseguibile carica e che il bundler NSIS (a differenza di quello MSI)
; non copia da solo accanto a spinlab.exe.
; il percorso si fissa qui: dentro la macro __FILEDIR__ indicherebbe la cartella dello script generato da Tauri
!define SPINLAB_RELEASE "${__FILEDIR__}\..\target\release"

!macro NSIS_HOOK_POSTINSTALL
  File "/oname=$INSTDIR\DirectML.dll" "${SPINLAB_RELEASE}\DirectML.dll"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  Delete "$INSTDIR\DirectML.dll"
!macroend
