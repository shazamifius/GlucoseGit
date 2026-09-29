; Attendre la fin des processus qui tiennent un fichier (fiche 48), inclus par glucose.nsi.
;
; Un Glucose encore ouvert tient son glucose.exe : l'updater de Tauri lance l'installeur puis
; quitte sans dire son numéro, celui de Glucose Rust fait de même, et quelqu'un peut installer à
; la main Glucose ouvert. On demande au Gestionnaire de redémarrage de Windows quels processus
; tiennent ces fichiers, et on attend exactement leur fin — aucune durée n'est choisie, et rien
; n'est fermé de force : un Glucose ouvert à la main peut porter du travail. (L'installeur de
; Tauri, lui, les tue : tauri-bundler, nsis/utils.nsh, CheckIfAppIsRunning.)
;
; Sans Gestionnaire de redémarrage, rien n'est attendu : un fichier encore tenu se dit alors par
; la question habituelle de NSIS, « Réessayer ».
;
; Utilise $0 à $9 et $R0.

!include LogicLib.nsh
!include Win\RestartManager.nsh

!macro AttendreCeuxQuiTiennent premier second
  !insertmacro RestartManager_StartSession $R0
  ${If} $R0 != ""
    !insertmacro RestartManager_RegisterFile $R0 "${premier}"
    !insertmacro RestartManager_RegisterFile $R0 "${second}"
    ; Un premier appel sans tableau : ERROR_MORE_DATA (234) et le nombre, s'il y en a.
    System::Call 'RSTRTMGR::RmGetList(i R0, *i .r1, *i 0 r2, p 0, *i .r3) i .r0'
    ${If} $0 = 234
      ; RM_PROCESS_INFO : RM_UNIQUE_PROCESS (12 octets, le numéro d'abord), le nom de
      ; l'application (256 caractères), celui du service (64), quatre entiers : 668 octets.
      IntOp $4 $1 * 668
      System::Alloc $4
      Pop $5
      System::Call 'RSTRTMGR::RmGetList(i R0, *i .r1, *i r1 r2, p r5, *i .r3) i .r0'
      ${If} $0 = 0
        DetailPrint "Glucose est encore ouvert : l'installation reprend dès qu'il est fermé."
        StrCpy $6 0
        ${DoWhile} $6 < $2
          IntOp $7 $6 * 668
          IntPtrOp $7 $5 + $7
          System::Call '*$7(i .r8)'
          ; SYNCHRONIZE : de quoi attendre, rien d'autre.
          System::Call 'kernel32::OpenProcess(i 0x00100000, i 0, i r8) p .r9'
          ${If} $9 P<> 0
            System::Call 'kernel32::WaitForSingleObject(p r9, i -1) i'
            System::Call 'kernel32::CloseHandle(p r9)'
          ${EndIf}
          IntOp $6 $6 + 1
        ${Loop}
      ${EndIf}
      System::Free $5
    ${EndIf}
    !insertmacro RestartManager_EndSession $R0
  ${EndIf}
!macroend
