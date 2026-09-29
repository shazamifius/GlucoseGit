<#
  La vie entière de Glucose sous Windows, éprouvée sur une machine jetable de GitHub (fiche 48).

  1. Glucose Tauri s'installe, comme chez ses utilisateurs. On plante des données : celles de
     Glucose Rust dans %LOCALAPPDATA%\Glucose — le dossier même où Tauri s'installe —, une image
     du magasin de Tauri, un document ; et l'épingle que Windows crée quand on épingle Glucose.
  2. La bascule, Glucose Tauri OUVERT : l'installeur démarre avec les arguments de l'updater de
     Tauri (/P /UPDATE ; sans /R, rien ne se relance ici). Il doit attendre que Glucose Tauri se
     ferme, puis le remplacer : ses deux fichiers retirés ; raccourcis, épingle et entrée de la
     liste des programmes repris, à l'identité com.glucose.app.
  3. La mise à jour : un serveur local sert un latest.json signé par le signataire de Tauri. Un
     installeur altéré est refusé sans rien poser ; l'installeur intact installe la version N+1.
  4. La désinstallation : le programme, ses raccourcis et son entrée partent ; rien d'autre.
  À chaque pas, les données plantées sont intactes, octet pour octet.

  ELLE NE SE LANCE QUE SUR UNE MACHINE JETABLE DE GITHUB : elle installe et désinstalle dans le
  compte courant, là où l'utilisateur garde ses brouillons et sa boîte noire. Ailleurs, elle
  refuse avant de toucher à quoi que ce soit.
#>
param(
    [Parameter(Mandatory)] [string] $Tauri,
    [Parameter(Mandatory)] [string] $Installeur,
    [Parameter(Mandatory)] [string] $Version,
    [Parameter(Mandatory)] [string] $Serveur,
    [Parameter(Mandatory)] [string] $VersionSuivante
)

if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') {
    Write-Host 'refus : cette epreuve installe et desinstalle dans le compte courant ; elle ne se lance que sur une machine jetable de GitHub'
    exit 2
}
$ErrorActionPreference = 'Stop'

$donnees = Join-Path $env:LOCALAPPDATA 'Glucose'
$programme = Join-Path $env:LOCALAPPDATA 'Programs\Glucose'
$exe = Join-Path $programme 'glucose.exe'
$magasin = Join-Path $env:APPDATA 'com.glucose.app'
$cle = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Glucose'
$menu = Join-Path ([Environment]::GetFolderPath('Programs')) 'Glucose.lnk'
$bureau = Join-Path ([Environment]::GetFolderPath('Desktop')) 'Glucose.lnk'
$epingle = Join-Path $env:APPDATA 'Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\Glucose.lnk'
$raccourcis = @($menu, $bureau, $epingle)

function Echouer([string] $message) {
    Write-Host "::error::$message"
    exit 1
}

function Constater([bool] $vrai, [string] $message) {
    if (-not $vrai) { Echouer $message }
    Write-Host "  ok : $message"
}

# Lance un programme et attend sa fin ; rend le processus (et son code de sortie).
function Lancer([string] $programme, [string[]] $arguments) {
    $p = Start-Process -FilePath $programme -ArgumentList $arguments -PassThru
    $null = $p.Handle
    if (-not $p.WaitForExit(600000)) { Echouer "$programme ne finit pas" }
    $p
}

# Attend les processus qu'un processus a lancés : un désinstalleur NSIS se recopie et se relance,
# une mise à jour lance l'installeur puis se ferme.
function AttendreSesEnfants($parent) {
    Get-CimInstance Win32_Process -Filter "ParentProcessId=$($parent.Id)" | ForEach-Object {
        $enfant = Get-Process -Id $_.ProcessId -ErrorAction SilentlyContinue
        if ($enfant) {
            $null = $enfant.Handle
            if (-not $enfant.WaitForExit(600000)) { Echouer "$($enfant.Name) ne finit pas" }
        }
    }
}

# Ce qu'un programme écrit sur sa sortie, et son code.
function Demander([string] $programme, [string] $argument) {
    $sortie = New-TemporaryFile
    $p = Start-Process -FilePath $programme -ArgumentList $argument -RedirectStandardOutput $sortie -PassThru
    $null = $p.Handle
    if (-not $p.WaitForExit(600000)) { Echouer "$programme $argument ne finit pas" }
    [pscustomobject]@{ Code = $p.ExitCode; Sortie = (Get-Content $sortie -Raw).Trim(); Processus = $p }
}

$coquille = New-Object -ComObject WScript.Shell
function Cible([string] $lnk) { $coquille.CreateShortcut($lnk).TargetPath }

# L'identité s'écrit dans le raccourci, en UTF-16 : on la cherche dans ses octets.
$motif = [BitConverter]::ToString([Text.Encoding]::Unicode.GetBytes('com.glucose.app'))
function PorteLIdentite([string] $lnk) {
    [BitConverter]::ToString([IO.File]::ReadAllBytes($lnk)).Contains($motif)
}

function Planter([string] $chemin, [int] $taille) {
    New-Item -ItemType Directory -Force (Split-Path $chemin) | Out-Null
    $octets = [byte[]]::new($taille)
    [Security.Cryptography.RandomNumberGenerator]::Fill($octets)
    [IO.File]::WriteAllBytes($chemin, $octets)
}

function Empreinte([string] $chemin) { (Get-FileHash $chemin -Algorithm SHA256).Hash }

function Intact([string] $moment) {
    $abimes = @($empreintes.Keys | Where-Object { -not (Test-Path $_) -or (Empreinte $_) -ne $empreintes[$_] })
    Constater ($abimes.Count -eq 0) "$moment : les $($empreintes.Count) fichiers plantés sont intacts $($abimes -join ', ')"
    $disparus = @($presents | Where-Object { -not (Test-Path $_) })
    Constater ($disparus.Count -eq 0) "$moment : rien n'a disparu de %LOCALAPPDATA%\Glucose $($disparus -join ', ')"
}

Write-Host '== 1. Glucose Tauri, installé et utilisé'
$p = Lancer $Tauri @('/S')
Constater ($p.ExitCode -eq 0) "l'installeur de Glucose Tauri finit sans erreur"
Constater (Test-Path "$donnees\glucose.exe") 'Glucose Tauri est installé dans %LOCALAPPDATA%\Glucose'
Write-Host "  (Glucose Tauri $((Get-ItemProperty $cle).DisplayVersion))"
Constater ((Cible $menu) -eq "$donnees\glucose.exe") 'son raccourci du menu Démarrer mène à lui'
Constater (PorteLIdentite $menu) "il y pose l'identité com.glucose.app"
Constater (Test-Path $bureau) 'il a posé un raccourci sur le bureau'
# Ce que Windows fait quand on épingle Glucose : une copie du raccourci, là.
New-Item -ItemType Directory -Force (Split-Path $epingle) | Out-Null
Copy-Item $menu $epingle

$documents = [Environment]::GetFolderPath('MyDocuments')
$plantes = @(
    "$donnees\brouillons\brouillon-essai.glucose",
    "$donnees\brouillons\recuperation\essai-0-1.fin",
    "$donnees\boite-noire\session-essai.jsonl",
    "$donnees\apercus\v2\essai.apercu",
    "$donnees\dernier-document.txt",
    "$magasin\assets\essai.png",
    "$documents\essai-tauri.glucose"
)
foreach ($f in $plantes) { Planter $f 65536 }
$empreintes = @{}
foreach ($f in $plantes) { $empreintes[$f] = Empreinte $f }
$presents = @(Get-ChildItem $donnees -Recurse -File |
    Where-Object { -not ($_.DirectoryName -eq $donnees -and $_.Name -in 'glucose.exe', 'uninstall.exe') } |
    ForEach-Object { $_.FullName })

Write-Host '== 2. La bascule, Glucose Tauri ouvert'
$ouvert = Start-Process "$donnees\glucose.exe" -PassThru
Start-Sleep -Seconds 10
Constater (-not $ouvert.HasExited) 'Glucose Tauri est ouvert'
$bascule = Start-Process $Installeur -ArgumentList '/P', '/UPDATE' -PassThru
$null = $bascule.Handle
# Une fenêtre d'observation : l'installeur finit en quelques secondes s'il n'attend pas.
Start-Sleep -Seconds 20
Constater (-not $bascule.HasExited) "l'installeur attend que Glucose Tauri se ferme"
Constater (-not (Test-Path $exe)) "il n'a rien posé avant"
Constater (Test-Path "$donnees\glucose.exe") "ni rien retiré"
Stop-Process -Id $ouvert.Id
if (-not $bascule.WaitForExit(600000)) { Echouer "l'installeur ne finit pas une fois Glucose Tauri fermé" }
Constater ($bascule.ExitCode -eq 0) "puis il s'installe sans erreur"
Constater (Test-Path $exe) 'Glucose Rust est installé dans %LOCALAPPDATA%\Programs\Glucose'
$v = Demander $exe '--version'
Constater ($v.Sortie -eq "Glucose $Version") "il dit sa version : $($v.Sortie)"
Constater (-not (Test-Path "$donnees\glucose.exe")) "l'exécutable de Glucose Tauri est retiré"
Constater (-not (Test-Path "$donnees\uninstall.exe")) 'son désinstalleur aussi'
Intact 'après la bascule'
$entree = Get-ItemProperty $cle
Constater ($entree.DisplayVersion -eq $Version) "la liste des programmes dit $($entree.DisplayVersion)"
Constater ($entree.UninstallString -eq "`"$programme\uninstall.exe`"") 'et désinstalle Glucose Rust'
Constater ($null -eq $entree.MainBinaryName) 'sans les valeurs que Tauri y laissait'
foreach ($lnk in $raccourcis) {
    Constater ((Cible $lnk) -eq $exe) "$lnk mène à Glucose Rust"
    Constater (PorteLIdentite $lnk) "$lnk porte l'identité com.glucose.app"
}

Write-Host '== 3. La mise à jour'
$serveurHttp = Start-Process python -ArgumentList '-m', 'http.server', '8765', '--bind', '127.0.0.1', '--directory', $Serveur -PassThru -WindowStyle Hidden
$pret = $false
foreach ($essai in 1..100) {
    try { Invoke-WebRequest 'http://127.0.0.1:8765/latest.json' -UseBasicParsing | Out-Null; $pret = $true; break }
    catch { Start-Sleep -Milliseconds 200 }
}
Constater $pret 'le serveur local sert latest.json'
$nom = "Glucose_${VersionSuivante}_x64-setup.exe"
$servi = Join-Path $Serveur $nom
$bon = Join-Path $env:RUNNER_TEMP $nom
Copy-Item $servi $bon
$octets = [IO.File]::ReadAllBytes($bon)
$octets[$octets.Length - 1] = $octets[$octets.Length - 1] -bxor 0xFF
[IO.File]::WriteAllBytes($servi, $octets)
$r = Demander $exe '--mettre-a-jour'
AttendreSesEnfants $r.Processus
Constater ($r.Code -ne 0) "un installeur altéré est refusé : $($r.Sortie)"
Constater ((Demander $exe '--version').Sortie -eq "Glucose $Version") 'rien ne change'
Constater (-not (Test-Path "$donnees\mises-a-jour\*")) "rien n'est posé sur le disque"
Copy-Item $bon $servi -Force
$r = Demander $exe '--mettre-a-jour'
AttendreSesEnfants $r.Processus
Constater ($r.Code -eq 0) "l'installeur signé est accepté : $($r.Sortie)"
Constater ((Demander $exe '--version').Sortie -eq "Glucose $VersionSuivante") "Glucose est passé à $VersionSuivante"
$pose = Get-ChildItem "$donnees\mises-a-jour" -File | Select-Object -First 1
Constater ($null -ne $pose -and (Empreinte $pose.FullName) -eq (Empreinte $bon)) "l'installeur posé est exactement celui qui est signé"
Constater ((Get-ItemProperty $cle).DisplayVersion -eq $VersionSuivante) "la liste des programmes dit $VersionSuivante"
Intact 'après la mise à jour'
Stop-Process -Id $serveurHttp.Id

Write-Host '== 4. La désinstallation'
$d = Lancer "$programme\uninstall.exe" @('/S')
AttendreSesEnfants $d
Constater (-not (Test-Path $programme)) 'le dossier du programme est retiré'
foreach ($lnk in $raccourcis) { Constater (-not (Test-Path $lnk)) "$lnk est retiré" }
Constater (-not (Test-Path $cle)) "l'entrée de la liste des programmes aussi"
Intact 'après la désinstallation'
Write-Host '== La vie entière de Glucose sous Windows est éprouvée.'
