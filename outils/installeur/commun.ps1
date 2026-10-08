# Ce que partagent les épreuves de Windows (fiche 48) : eprouver.ps1 et repetition.ps1.
# Chacune vérifie d'abord, elle-même, qu'elle tourne sur une machine jetable de GitHub — avant
# d'inclure ce fichier.

$ErrorActionPreference = 'Stop'

$donnees = Join-Path $env:LOCALAPPDATA 'Glucose'
$programme = Join-Path $env:LOCALAPPDATA 'Programs\Glucose'
$exe = Join-Path $programme 'glucose.exe'
$magasin = Join-Path $env:APPDATA 'com.glucose.app'
$documents = [Environment]::GetFolderPath('MyDocuments')
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

# Plante ces fichiers et retient leurs empreintes.
function PlanterTout([string[]] $chemins) {
    $table = @{}
    foreach ($f in $chemins) {
        Planter $f 65536
        $table[$f] = Empreinte $f
    }
    $table
}

function Intacts($empreintes, [string] $moment) {
    $abimes = @($empreintes.Keys | Where-Object { -not (Test-Path $_) -or (Empreinte $_) -ne $empreintes[$_] })
    Constater ($abimes.Count -eq 0) "$moment : les $($empreintes.Count) fichiers plantés sont intacts $($abimes -join ', ')"
}

# Ce qu'un utilisateur a sur sa machine le jour de la bascule, planté une fois Glucose Tauri
# installé : les données de Glucose Rust dans %LOCALAPPDATA%\Glucose — le dossier même où Tauri
# s'installe —, une image du magasin de Tauri, un document. Rend les empreintes, et ce que
# %LOCALAPPDATA%\Glucose contient hors des deux fichiers de Tauri.
function PlanterSesDonnees {
    $empreintes = PlanterTout @(
        "$donnees\brouillons\brouillon-essai.glucose",
        "$donnees\brouillons\recuperation\essai-0-1.fin",
        "$donnees\journal-technique\session-essai.jsonl",
        "$donnees\apercus\v2\essai.apercu",
        "$donnees\dernier-document.txt",
        "$magasin\assets\essai.png",
        "$documents\essai-tauri.glucose"
    )
    $presents = @(Get-ChildItem $donnees -Recurse -File |
        Where-Object { -not ($_.DirectoryName -eq $donnees -and $_.Name -in 'glucose.exe', 'uninstall.exe') } |
        ForEach-Object { $_.FullName })
    [pscustomobject]@{ Empreintes = $empreintes; Presents = $presents }
}

# Ses données sont intactes, et rien n'a disparu de %LOCALAPPDATA%\Glucose.
function Intact($sesDonnees, [string] $moment) {
    Intacts $sesDonnees.Empreintes $moment
    $disparus = @($sesDonnees.Presents | Where-Object { -not (Test-Path $_) })
    Constater ($disparus.Count -eq 0) "$moment : rien n'a disparu de %LOCALAPPDATA%\Glucose $($disparus -join ', ')"
}

# Le serveur local des versions, prêt à répondre. Il garde le journal de ce qu'on lui demande :
# c'est ce qui dit, si une épreuve tombe, jusqu'où un programme de mise à jour est allé.
$journalDuServeur = Join-Path $env:RUNNER_TEMP 'serveur-requetes.txt'
function Servir([string] $dossier) {
    $serveur = Start-Process python -ArgumentList '-u', '-m', 'http.server', '8765', '--bind', '127.0.0.1', '--directory', $dossier -PassThru -NoNewWindow -RedirectStandardOutput (Join-Path $env:RUNNER_TEMP 'serveur-sortie.txt') -RedirectStandardError $journalDuServeur
    foreach ($essai in 1..100) {
        try { Invoke-WebRequest 'http://127.0.0.1:8765/latest.json' -UseBasicParsing | Out-Null; return $serveur }
        catch { Start-Sleep -Milliseconds 200 }
    }
    Echouer 'le serveur local ne sert pas latest.json'
}

# Ce que le serveur a servi jusqu'ici.
function DireCeQueLeServeurAServi {
    Write-Host '  -- ce que le serveur local a servi :'
    if (Test-Path $journalDuServeur) { Get-Content $journalDuServeur | ForEach-Object { Write-Host "     $_" } }
}

# Chaque raccourci mène à Glucose Rust, et porte son identité.
function RaccourcisRepris {
    foreach ($lnk in $raccourcis) {
        Constater ((Cible $lnk) -eq $exe) "$lnk mène à Glucose Rust"
        Constater (PorteLIdentite $lnk) "$lnk porte l'identité com.glucose.app"
    }
}
