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
. (Join-Path $PSScriptRoot 'commun.ps1')

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

$sesDonnees = PlanterSesDonnees

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
Intact $sesDonnees 'après la bascule'
$entree = Get-ItemProperty $cle
Constater ($entree.DisplayVersion -eq $Version) "la liste des programmes dit $($entree.DisplayVersion)"
Constater ($entree.UninstallString -eq "`"$programme\uninstall.exe`"") 'et désinstalle Glucose Rust'
Constater ($null -eq $entree.MainBinaryName) 'sans les valeurs que Tauri y laissait'
RaccourcisRepris

Write-Host '== 3. La mise à jour'
$serveurHttp = Servir $Serveur
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
Constater ($r.Sortie -match 'signature') 'pour sa signature'
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
Intact $sesDonnees 'après la mise à jour'
Stop-Process -Id $serveurHttp.Id

Write-Host '== 4. La désinstallation'
$d = Lancer "$programme\uninstall.exe" @('/S')
AttendreSesEnfants $d
Constater (-not (Test-Path $programme)) 'le dossier du programme est retiré'
foreach ($lnk in $raccourcis) { Constater (-not (Test-Path $lnk)) "$lnk est retiré" }
Constater (-not (Test-Path $cle)) "l'entrée de la liste des programmes aussi"
Intact $sesDonnees 'après la désinstallation'
Write-Host '== La vie entière de Glucose sous Windows est éprouvée.'
