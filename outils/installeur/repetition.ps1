<#
  La répétition générale de la bascule, sur une machine jetable de GitHub (fiche 48).

  Glucose Tauri — reconstruit depuis sa source publiée (v1.0.2-beta.1), avec une clé d'essai et
  un serveur local (preparer_tauri.py) — se met à jour PAR SON PROPRE UPDATER : il trouve la
  version, la télécharge, vérifie sa signature par sa clé, lance l'installeur avec ses arguments
  (/P /R /UPDATE /ARGS), et se ferme. Seul le clic de son popup est fait par le programme : les
  deux appels que fait son bouton. Et Glucose Rust doit arriver : installé, relancé par son
  installeur, vivant — les données de l'utilisateur intactes.

  C'est ce qui arrivera chez ses utilisateurs Windows, à la clé près.

  ELLE NE SE LANCE QUE SUR UNE MACHINE JETABLE DE GITHUB : elle installe dans le compte courant.
#>
param(
    [Parameter(Mandatory)] [string] $Tauri,
    [Parameter(Mandatory)] [string] $Serveur,
    [Parameter(Mandatory)] [string] $Version
)

if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') {
    Write-Host 'refus : cette epreuve installe dans le compte courant ; elle ne se lance que sur une machine jetable de GitHub'
    exit 2
}
. (Join-Path $PSScriptRoot 'commun.ps1')

Write-Host '== 1. Glucose Tauri, tel que ses utilisateurs l''ont — à la clé près'
$p = Lancer $Tauri @('/S')
Constater ($p.ExitCode -eq 0) "l'installeur de Glucose Tauri finit sans erreur"
Constater (Test-Path "$donnees\glucose.exe") 'Glucose Tauri est installé dans %LOCALAPPDATA%\Glucose'
$sesDonnees = PlanterSesDonnees
New-Item -ItemType Directory -Force (Split-Path $epingle) | Out-Null
Copy-Item $menu $epingle

Write-Host '== 2. Son updater fait la bascule'
$serveurHttp = Servir $Serveur
$tauri = Start-Process "$donnees\glucose.exe" -PassThru
$null = $tauri.Handle
# Glucose Rust doit apparaître : posé par l'installeur, puis relancé par lui (/R). On regarde
# les processus, faute de pouvoir regarder l'écran.
$limite = (Get-Date).AddMinutes(10)
$rust = $null
while (-not $rust -and (Get-Date) -lt $limite) {
    Start-Sleep -Seconds 2
    $rust = Get-Process glucose -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -eq $exe } | Select-Object -First 1
}
if (-not $rust) {
    # Ce qui dit jusqu'où l'updater de Tauri est allé : ce qu'il a demandé au serveur, ce qu'il a
    # écrit dans le dossier temporaire, ce qui tourne encore.
    DireCeQueLeServeurAServi
    Write-Host "  -- Glucose Tauri fini : $($tauri.HasExited)"
    Get-ChildItem $env:TEMP -Directory -Filter 'Glucose-*' -ErrorAction SilentlyContinue |
        ForEach-Object { Write-Host "  -- dossier de l'updater : $($_.FullName)"; Get-ChildItem $_.FullName | ForEach-Object { Write-Host "     $($_.Name) $($_.Length)" } }
    Get-Process glucose, *setup*, *installer* -ErrorAction SilentlyContinue |
        ForEach-Object { Write-Host "  -- processus : $($_.Name) $($_.Path)" }
    Echouer 'Glucose Rust n''est pas arrivé en dix minutes'
}
$null = $rust.Handle
Write-Host '  ok : Glucose Rust a été installé, puis relancé par son installeur'
DireCeQueLeServeurAServi
Constater $tauri.HasExited 'Glucose Tauri s''est fermé de lui-même'
Start-Sleep -Seconds 15
Constater (-not $rust.HasExited) 'Glucose Rust vit, quinze secondes après'
Constater ((Demander $exe '--version').Sortie -eq "Glucose $Version") "il est la version $Version"
Constater (-not (Test-Path "$donnees\glucose.exe")) "l'exécutable de Glucose Tauri est retiré"
Constater (-not (Test-Path "$donnees\uninstall.exe")) 'son désinstalleur aussi'
Constater ((Get-ItemProperty $cle).DisplayVersion -eq $Version) "la liste des programmes dit $Version"
RaccourcisRepris
Intact $sesDonnees 'après la bascule'
Stop-Process -Id $rust.Id
Stop-Process -Id $serveurHttp.Id
Write-Host '== La bascule, par l''updater de Glucose Tauri lui-même, est éprouvée.'
