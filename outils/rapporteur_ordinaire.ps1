<#
  Le rapporteur d'erreurs de Windows, réglé comme sur un poste ordinaire (fiche 49 § 1).

  L'épreuve « un vrai plantage se lit dans le journal » (plateforme::journal) fait tomber un
  processus et exige que Windows l'ait noté. Chez ses utilisateurs, le rapporteur est allumé et
  note (Disabled = 0, LoggingDisabled = 0 : les valeurs par défaut). Ce script dit d'abord ce
  qu'il trouve, puis garantit ces valeurs — et l'absence de fenêtre (DontShowUI = 1), qu'aucune
  machine sans écran ne doit attendre. (Le 30/09, la machine de GitHub les avait déjà : ce
  n'était pas la cause du silence — c'était le mode d'erreur hérité, que l'épreuve remet à son
  défaut.)
  learn.microsoft.com/windows/win32/wer/wer-settings

  NE SE LANCE QUE SUR UNE MACHINE JETABLE DE GITHUB : il change un réglage de la machine.
#>
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') {
    Write-Host 'refus : ce script regle le rapporteur de la machine ; il ne se lance que sur une machine jetable de GitHub'
    exit 2
}
$ErrorActionPreference = 'Stop'

$cles = @(
    'HKLM:\SOFTWARE\Microsoft\Windows\Windows Error Reporting',
    'HKCU:\Software\Microsoft\Windows\Windows Error Reporting',
    'HKLM:\SOFTWARE\Policies\Microsoft\Windows\Windows Error Reporting'
)
$valeurs = 'Disabled', 'LoggingDisabled', 'DontShowUI'

Write-Host '== Ce que la machine avait'
foreach ($cle in $cles) {
    if (Test-Path $cle) {
        $lu = Get-ItemProperty $cle
        Write-Host "  $cle : $(($valeurs | ForEach-Object { "$_=$($lu.$_)" }) -join ', ')"
    } else {
        Write-Host "  $cle : absente"
    }
}
Write-Host "  service WerSvc : $((Get-Service WerSvc).Status), $((Get-Service WerSvc).StartType)"

Write-Host '== Ce qu''un poste ordinaire a, sans fenetre'
foreach ($cle in $cles) {
    if (-not (Test-Path $cle)) { continue }
    Set-ItemProperty $cle -Name Disabled -Value 0 -Type DWord
    Set-ItemProperty $cle -Name LoggingDisabled -Value 0 -Type DWord
    Set-ItemProperty $cle -Name DontShowUI -Value 1 -Type DWord
    Write-Host "  $cle : Disabled=0, LoggingDisabled=0, DontShowUI=1"
}
