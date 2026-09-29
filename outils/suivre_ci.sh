#!/bin/bash
# Suit l'exécution de la CI pour un commit donné, jusqu'à ce qu'elle finisse. Écrit l'état dans
# un fichier ; ne montre jamais le jeton.
# Usage : bash outils/suivre_ci.sh <empreinte COMPLETE du commit> <fichier de sortie>
# (fiche 46 § 5). L'empreinte courte ne trouve rien.
cd "$(dirname "$0")/.."
SHA="$1"; SORTIE="$2"
T=$(git remote get-url origin | sed -n 's#https://\([^@]*\)@github.com.*#\1#p' | sed 's/^[^:]*://')
api() { curl -s -H "Authorization: Bearer $T" -H "Accept: application/vnd.github+json" "https://api.github.com/repos/shazamifius/GlucoseGit/$1"; }
for i in $(seq 1 120); do
  RUN=$(api "actions/runs?head_sha=$SHA&per_page=1" | python -c "import json,sys; r=json.load(sys.stdin)['workflow_runs']; print(r[0]['id'], r[0]['status'], r[0]['conclusion']) if r else print('')")
  if [ -n "$RUN" ]; then
    set -- $RUN; ID=$1; ETAT=$2
    api "actions/runs/$ID/jobs" | python -c "
import json,sys
d=json.load(sys.stdin)
for j in d['jobs']:
    rate=[s['name'] for s in j['steps'] if s['conclusion'] not in ('success','skipped',None)]
    print(j['name'], '|', j['status'], j['conclusion'], '| etapes en echec:', rate, '| id', j['id'])
" > "$SORTIE"
    echo "run $ID $ETAT $3" >> "$SORTIE"
    [ "$ETAT" = "completed" ] && exit 0
  fi
  sleep 60
done
echo "delai depasse" >> "$SORTIE"
