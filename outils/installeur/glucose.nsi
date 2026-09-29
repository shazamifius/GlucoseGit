; L'installeur de Glucose Rust sous Windows (fiche 48).
;
; Construit par outils/installeur/construire.py :
;   makensis /DVERSION=2.0.1 /DVERSION_NUMERIQUE=2.0.1.0 /DEXE=...\glucose-desktop.exe
;            /DSORTIE=...\Glucose_2.0.1_x64-setup.exe glucose.nsi
;
; Ce qu'il promet, et que la vérification de GitHub éprouve :
; * il s'installe pour l'utilisateur seul, sans droits d'administrateur, dans le dossier que
;   Windows réserve à cela (FOLDERID_UserProgramFiles : %LOCALAPPDATA%\Programs\Glucose) ;
; * il attend EXACTEMENT la fin de tout Glucose encore ouvert — l'ancien de Tauri comme celui-ci —,
;   sans jamais le fermer de force : c'est Windows qui dit quels processus tiennent glucose.exe ;
; * lancé par une mise à jour (/P /R /UPDATE, les arguments de l'updater de Tauri), il s'installe
;   sans question, puis relance Glucose ;
; * il remplace Glucose Tauri sans rien perdre : il retire ses DEUX fichiers — glucose.exe et
;   uninstall.exe, ceux, et seulement ceux, que son propre désinstalleur retire —, reprend son
;   entrée dans la liste des programmes, ses raccourcis et son identité ; le reste de %LOCALAPPDATA%\Glucose —
;   les brouillons, la boîte noire, les aperçus de Glucose Rust — ne se touche jamais, pas plus
;   que le magasin d'images de Tauri (%APPDATA%\com.glucose.app), que ses documents demandent ;
; * son désinstalleur ne retire que ce que l'installeur a posé. Aucun RMDir /r, nulle part.

Unicode true
ManifestDPIAware true
SetCompressor /SOLID lzma
RequestExecutionLevel user

!include MUI2.nsh
!include FileFunc.nsh
!include LogicLib.nsh
!include Win\COM.nsh
!include Win\Propkey.nsh
!include attendre.nsh

!ifndef VERSION
  !error "VERSION manque : /DVERSION=2.0.1"
!endif
!ifndef VERSION_NUMERIQUE
  !error "VERSION_NUMERIQUE manque : /DVERSION_NUMERIQUE=2.0.1.0"
!endif
!ifndef EXE
  !error "EXE manque : le chemin de glucose-desktop.exe"
!endif
!ifndef SORTIE
  !define SORTIE "Glucose_${VERSION}_x64-setup.exe"
!endif

; La même clé que Glucose Tauri : la bascule reprend son entrée dans la liste des programmes.
!define CLE "Software\Microsoft\Windows\CurrentVersion\Uninstall\Glucose"
!define EPINGLE "$APPDATA\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\Glucose.lnk"
; L'identité de Glucose pour Windows (AppUserModelID) : celle que Glucose Tauri a posée sur ses
; raccourcis, et que le programme déclare à son lancement (plateforme::identite). La même sur le
; processus et sur chaque raccourci, sans quoi la barre des tâches sépare la fenêtre de son
; épingle (learn.microsoft.com/windows/win32/shell/appids).
!define IDENTITE "com.glucose.app"

Name "Glucose"
OutFile "${SORTIE}"
BrandingText "Glucose ${VERSION}"
VIProductVersion "${VERSION_NUMERIQUE}"
VIAddVersionKey /LANG=0 "ProductName" "Glucose"
VIAddVersionKey /LANG=0 "ProductVersion" "${VERSION}"
VIAddVersionKey /LANG=0 "FileVersion" "${VERSION}"
VIAddVersionKey /LANG=0 "FileDescription" "Installeur de Glucose"
VIAddVersionKey /LANG=0 "LegalCopyright" "Glucose"

!define MUI_ICON "glucose.ico"
!define MUI_UNICON "glucose.ico"
!define MUI_FINISHPAGE_RUN "$INSTDIR\glucose.exe"

Var Passif
Var MiseAJour
Var Relancer

!define MUI_PAGE_CUSTOMFUNCTION_PRE PasserSiPassif
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_INSTFILES
!define MUI_PAGE_CUSTOMFUNCTION_PRE PasserSiPassif
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "French"
!insertmacro MUI_LANGUAGE "English"

; Ce que la ligne de commande demande, comme chez Glucose Tauri : /P sans question, /R relancer,
; /UPDATE une mise à jour.
Function .onInit
  ${GetParameters} $0
  ClearErrors
  ${GetOptions} $0 "/P" $1
  ${IfNot} ${Errors}
    StrCpy $Passif 1
  ${EndIf}
  ClearErrors
  ${GetOptions} $0 "/UPDATE" $1
  ${IfNot} ${Errors}
    StrCpy $MiseAJour 1
  ${EndIf}
  ClearErrors
  ${GetOptions} $0 "/R" $1
  ${IfNot} ${Errors}
    StrCpy $Relancer 1
  ${EndIf}
  ; Le dossier des programmes d'un seul utilisateur, tel que Windows le connaît — sauf /D.
  ${If} $INSTDIR == ""
    System::Call 'shell32::SHGetKnownFolderPath(g "{5CD7AEE2-2219-4A67-B85D-6C9CE15660CB}", i 0x8000, p 0, *p .r1) i .r0'
    ${If} $0 == 0
      System::Call '*$1(&w${NSIS_MAX_STRLEN} .r2)'
      System::Call 'ole32::CoTaskMemFree(p r1)'
      StrCpy $INSTDIR "$2\Glucose"
    ${Else}
      StrCpy $INSTDIR "$LOCALAPPDATA\Programs\Glucose"
    ${EndIf}
  ${EndIf}
FunctionEnd

Function PasserSiPassif
  ${If} $Passif == 1
    Abort
  ${EndIf}
FunctionEnd

; Tout Glucose encore ouvert — celui-ci, ou l'ancien de Tauri — se ferme avant qu'on touche à
; quoi que ce soit (attendre.nsh).
Function AttendreLesGlucoseOuverts
  !insertmacro AttendreCeuxQuiTiennent "$INSTDIR\glucose.exe" "$LOCALAPPDATA\Glucose\glucose.exe"
FunctionEnd

; Glucose Tauri s'installait dans le dossier même où Glucose Rust garde ses données. On retire
; ses deux fichiers, et le dossier seulement s'il est vide (RMDir sans /r) : ce qui y reste est à
; l'utilisateur. Un fichier encore tenu reste en place — un reste inoffensif, jamais une perte.
Function RetirerGlucoseTauri
  StrCpy $0 "$LOCALAPPDATA\Glucose"
  ${If} $INSTDIR == $0
    Return
  ${EndIf}
  ${If} ${FileExists} "$0\glucose.exe"
  ${OrIf} ${FileExists} "$0\uninstall.exe"
    DetailPrint "Glucose Tauri : ses deux fichiers se retirent, ses données restent."
    Delete "$0\glucose.exe"
    Delete "$0\uninstall.exe"
    RMDir "$0"
  ${EndIf}
FunctionEnd

; Pose l'identité de Glucose sur le raccourci $R9, comme l'installeur de Tauri le fait
; (tauri-bundler, nsis/utils.nsh, SetLnkAppUserModelId) : IShellLink, IPersistFile,
; IPropertyStore.
Function Estampiller
  !insertmacro ComHlpr_CreateInProcInstance ${CLSID_ShellLink} ${IID_IShellLink} r0 ""
  ${If} $0 P<> 0
    ${IUnknown::QueryInterface} $0 '("${IID_IPersistFile}",.r1)'
    ${If} $1 P<> 0
      ${IPersistFile::Load} $1 '("$R9", ${STGM_READWRITE})'
      ${IUnknown::QueryInterface} $0 '("${IID_IPropertyStore}",.r2)'
      ${If} $2 P<> 0
        System::Call 'Oleaut32::SysAllocString(w "${IDENTITE}") i.r3'
        System::Call '*${SYSSTRUCT_PROPERTYKEY}(${PKEY_AppUserModel_ID})p.r4'
        System::Call '*${SYSSTRUCT_PROPVARIANT}(${VT_BSTR},,&i4 $3)p.r5'
        ${IPropertyStore::SetValue} $2 '($4,$5)'
        System::Call 'Oleaut32::SysFreeString($3)'
        System::Free $4
        System::Free $5
        ${IPropertyStore::Commit} $2 ""
        ${IUnknown::Release} $2 ""
        ${IPersistFile::Save} $1 '("$R9",1)'
      ${EndIf}
      ${IUnknown::Release} $1 ""
    ${EndIf}
    ${IUnknown::Release} $0 ""
  ${EndIf}
FunctionEnd

; Un raccourci $R9 vers ce programme, à l'identité de Glucose.
Function Raccourci
  CreateShortCut "$R9" "$INSTDIR\glucose.exe" "" "$INSTDIR\glucose.ico"
  Call Estampiller
FunctionEnd

; Le menu Démarrer toujours ; le bureau à une première installation, ou là où il y en avait un ;
; l'épingle de la barre des tâches, si elle existe, pointe désormais sur ce programme.
Function Raccourcis
  StrCpy $R9 "$SMPROGRAMS\Glucose.lnk"
  Call Raccourci
  ${If} $MiseAJour != 1
  ${OrIf} ${FileExists} "$DESKTOP\Glucose.lnk"
    StrCpy $R9 "$DESKTOP\Glucose.lnk"
    Call Raccourci
  ${EndIf}
  ${If} ${FileExists} "${EPINGLE}"
    StrCpy $R9 "${EPINGLE}"
    Call Raccourci
  ${EndIf}
FunctionEnd

; L'entrée de la liste des programmes, écrite à neuf : celle de Glucose Tauri portait des
; valeurs que ce programme n'a pas (MainBinaryName, CurrentUser…).
Function Inscrire
  DeleteRegKey HKCU "${CLE}"
  WriteRegStr HKCU "${CLE}" "DisplayName" "Glucose"
  WriteRegStr HKCU "${CLE}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${CLE}" "Publisher" "Glucose"
  WriteRegStr HKCU "${CLE}" "DisplayIcon" "$\"$INSTDIR\glucose.ico$\""
  WriteRegStr HKCU "${CLE}" "InstallLocation" "$\"$INSTDIR$\""
  WriteRegStr HKCU "${CLE}" "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
  WriteRegStr HKCU "${CLE}" "QuietUninstallString" "$\"$INSTDIR\uninstall.exe$\" /S"
  WriteRegDWORD HKCU "${CLE}" "NoModify" 1
  WriteRegDWORD HKCU "${CLE}" "NoRepair" 1
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  WriteRegDWORD HKCU "${CLE}" "EstimatedSize" $0
FunctionEnd

Section "Glucose"
  SetShellVarContext current
  Call AttendreLesGlucoseOuverts
  SetOutPath "$INSTDIR"
  File "/oname=glucose.exe" "${EXE}"
  File "glucose.ico"
  WriteUninstaller "$INSTDIR\uninstall.exe"
  Call RetirerGlucoseTauri
  Call Raccourcis
  Call Inscrire
  ${If} $Passif == 1
  ${OrIf} $MiseAJour == 1
    SetAutoClose true
  ${EndIf}
  ${If} $Relancer == 1
    Exec '"$INSTDIR\glucose.exe"'
  ${EndIf}
SectionEnd

; Le désinstalleur ne retire que ce que l'installeur a posé. Les documents, les brouillons, la
; boîte noire et ce que Glucose a appris de la machine vivent dans %LOCALAPPDATA%\Glucose, et
; n'appartiennent qu'à l'utilisateur : rien ici ne les touche.
Section "Uninstall"
  SetShellVarContext current
  Delete "$INSTDIR\glucose.exe"
  Delete "$INSTDIR\glucose.ico"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  Delete "$SMPROGRAMS\Glucose.lnk"
  Delete "$DESKTOP\Glucose.lnk"
  Delete "${EPINGLE}"
  DeleteRegKey HKCU "${CLE}"
SectionEnd
