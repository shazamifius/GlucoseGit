#!/usr/bin/env bash
# **L'essai d'une page exportée** (EXPORT-HTML-1, fiche 115) : un Chromium sans fenêtre (Brave ici,
# sinon `chromium`), un profil jetable — jamais le sien —, puis `piloter.mjs` : la page ouverte, la
# flèche touchée, deux doigts qui pincent ; trois captures et ce qui s'est passé dans <sortie>.
#
#   bash outils/navigateur/essai_page.sh <page.html> <sortie> [identifiant d'une flèche]
set -u
page=$(realpath "${1:?la page}"); sortie=$(mkdir -p "${2:?le dossier}" && cd "$2" && pwd)
ici=$(cd "$(dirname "$0")" && pwd)
nav=$(command -v brave || command -v chromium) || { echo "ni brave ni chromium" >&2; exit 2; }
profil=$(mktemp -d); port=9333
"$nav" --headless=new --disable-gpu --no-first-run --user-data-dir="$profil" \
  --remote-debugging-port=$port about:blank > "$sortie/navigateur.txt" 2>&1 &
pid=$!
for _ in $(seq 50); do curl -s "http://127.0.0.1:$port/json" > /dev/null && break; sleep 0.2; done
timeout 60 node "$ici/piloter.mjs" $port "$page" "$sortie" "${3:-s-borne}" | tee "$sortie/essai.txt"
code=${PIPESTATUS[0]}
kill $pid 2>/dev/null; wait $pid 2>/dev/null; rm -rf "$profil"
exit "$code"
