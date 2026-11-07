#!/usr/bin/env python3
# **L'essai réel du domaine** (fiche 88) : le vrai Glucose sans fenêtre, de vrais fichiers, un client
# d'agent par la vraie porte. Rien ne s'ouvre sur son écran, et son dossier Glucose n'est jamais touché.
#
#   nix develop -c cargo build -p glucose-desktop --example porte_sans_fenetre
#   nix develop -c python3 -I outils/essai_domaine.py "$PWD" ~/.cache/glucose-bancs/d
#
# Le second chemin est vidé puis rempli. Code 0 : tout passe ; chaque vérification se lit sur la sortie.
import json,os,socket,subprocess,sys,time,shutil
RACINE,T=sys.argv[1],sys.argv[2]
shutil.rmtree(T,ignore_errors=True)
os.makedirs(T)
PROG=os.path.join(RACINE,'target/debug/examples/porte_sans_fenetre')
class Glucose:
    def __init__(s,nom):
        s.d=os.path.join(T,nom); os.makedirs(s.d)
        s.prise=os.path.join(s.d,'porte'); s.arret=os.path.join(s.d,'arret'); s.main=os.path.join(s.d,'images','main.jsonl')
        os.makedirs(os.path.join(s.d,'images')); open(s.main,'w').close()
        env=dict(os.environ,GLUCOSE_DOSSIER_ESSAI=os.path.join(s.d,'glucose'),TMPDIR=os.path.join(s.d,'tmp'))
        os.makedirs(env['TMPDIR']); os.makedirs(env['GLUCOSE_DOSSIER_ESSAI'])
        s.sortie=open(os.path.join(s.d,'sortie.txt'),'w')
        s.p=subprocess.Popen([PROG,s.prise,s.arret,os.path.join(s.d,'images')],stdout=s.sortie,stderr=subprocess.STDOUT,env=env)
        for _ in range(600):
            if os.path.exists(s.prise): break
            time.sleep(0.05)
        s.c=socket.socket(socket.AF_UNIX); s.c.connect(s.prise); s.f=s.c.makefile('rw',encoding='utf-8'); s.n=0
        s.rpc('initialize',{'protocolVersion':'2025-06-18','capabilities':{},'clientInfo':{'name':'essai','version':'1'}})
    def rpc(s,methode,params):
        s.n+=1
        s.f.write(json.dumps({'jsonrpc':'2.0','id':s.n,'method':methode,'params':params},ensure_ascii=False)+'\n'); s.f.flush()
        return json.loads(s.f.readline())
    def outil(s,nom,**a):
        r=s.rpc('tools/call',{'name':nom,'arguments':a})
        res=r.get('result',{})
        texte=res.get('content',[{}])[0].get('text','')
        s.dernier=len(texte)
        if res.get('isError') or 'error' in r: return ('REFUS',texte or json.dumps(r,ensure_ascii=False))
        return ('OK',json.loads(texte))
    def geste(s,**g):
        with open(s.main,'a') as f: f.write(json.dumps(g,ensure_ascii=False)+'\n')
        time.sleep(0.4)
    def attendre_lecture(s,nom='glucose_domaines'):
        for _ in range(200):
            e,r=s.outil(nom)
            if r.get('a_jour'): return r
            time.sleep(0.1)
        raise SystemExit('la lecture ne finit pas')
    def fin(s):
        open(s.arret,'w').close(); s.p.wait(timeout=120); s.c.close()
def titre(t): print('\n=== '+t)
ok=lambda c,m: print(('  OK   ' if c else '  FAUX ')+m) or c or fautes.append(m)
fautes=[]

titre('1. Deux fichiers, un domaine')
g=Glucose('un')
dossier_a=os.path.join(T,'projets','physique'); os.makedirs(dossier_a)
dossier_b=os.path.join(T,'projets','jeu'); os.makedirs(dossier_b)
a,b=os.path.join(dossier_a,'eau.glucose'),os.path.join(dossier_b,'moteur.glucose')
g.geste(enregistrer=a)
g.outil('glucose_poser_membrane',id='bulle',titre='Ce que je sais',x=0,y=0)
g.outil('glucose_poser_texte',id='mouille',texte="L'eau, ça mouille. C'est tout.",dans='bulle')
g.outil('glucose_poser_texte',id='h2o',texte='Eau = H2O. Deux hydrogènes, un oxygène.',x=900,y=0)
e,r=g.outil('glucose_poser_domaine',nouveau='Eau',ids=['mouille','h2o'],poids=1.0)
ok(e=='OK' and r['cree'],f'créer « Eau » sur deux nœuds : {r}')
g.geste(nouveau_document=True)
print('  questions du nouveau document :',open(os.path.join(g.d,'sortie.txt')).read().count('[main]'))
g.geste(enregistrer=b)
g.outil('glucose_poser_texte',id='vague',texte="L'eau dans les jeux vidéo : il faut du bruit et du displacement.",x=0,y=0)
r=g.attendre_lecture()
print('  table :',json.dumps(r['domaines'],ensure_ascii=False))
ok(any(d['nom']=='Eau' and d['noeuds']==2 for d in r['domaines']),'le fichier ouvert connaît « Eau » du fichier fermé')
e,q=g.outil('glucose_poser_domaine',nouveau='eau',ids=['vague'])
print('  question :',q)
ok(e=='REFUS' and 'existe déjà' in q and 'mouille' in q,'recréer « eau » : refusé, avec la question et un exemple')
e,r=g.outil('glucose_poser_domaine',domaine='eau',ids=['vague'],poids=0.7)
ok(e=='OK',f'même sujet : le nœud rejoint le domaine : {r}')
e,r=g.outil('glucose_domaines',domaine='eau')
pl=[(os.path.basename(n['fichier']),n['id'],n['poids']) for n in r['noeuds']]
print('  nœuds :',pl)
ok(len(pl)==3 and {x[0] for x in pl}=={'eau.glucose','moteur.glucose'},'TROIS BLOCS, DEUX FICHIERS, UN SEUL DOMAINE')
ok(g.outil('glucose_lire')[1]['noeuds'][0]['id']=='vague','rien ne s\'est ouvert à l\'écran : il regarde toujours son tableau')
e,r=g.outil('glucose_domaines',domaine='eau',entier=True)
ok(any(n.get('texte','').startswith('Eau = H2O') for n in r['noeuds']),'un agent lit le texte entier d\'un nœud du fichier fermé')
g.geste(panneau_domaines=True)

titre('2. Fermé puis relancé : rien de perdu')
g.fin()
print('  documents connus :',open(os.path.join(g.d,'glucose','documents-connus.txt')).read().split())
g.d2=g.d
# La relance : on garde le dossier de Glucose.
class Relance(Glucose):
    def __init__(s,base,nom):
        s.d=os.path.join(T,nom); os.makedirs(s.d)
        s.prise=os.path.join(s.d,'porte'); s.arret=os.path.join(s.d,'arret'); s.main=os.path.join(s.d,'images','main.jsonl')
        os.makedirs(os.path.join(s.d,'images')); open(s.main,'w').close()
        env=dict(os.environ,GLUCOSE_DOSSIER_ESSAI=os.path.join(base,'glucose'),TMPDIR=os.path.join(s.d,'tmp'))
        os.makedirs(env['TMPDIR'])
        s.sortie=open(os.path.join(s.d,'sortie.txt'),'w')
        s.p=subprocess.Popen([PROG,s.prise,s.arret,os.path.join(s.d,'images')],stdout=s.sortie,stderr=subprocess.STDOUT,env=env)
        for _ in range(600):
            if os.path.exists(s.prise): break
            time.sleep(0.05)
        s.c=socket.socket(socket.AF_UNIX); s.c.connect(s.prise); s.f=s.c.makefile('rw',encoding='utf-8'); s.n=0
        s.rpc('initialize',{'protocolVersion':'2025-06-18','capabilities':{},'clientInfo':{'name':'essai','version':'1'}})
g2=Relance(g.d,'deux')
time.sleep(0.5)
r=g2.attendre_lecture()
d=[x for x in r['domaines'] if x['nom']=='Eau']
ok(d and d[0]['noeuds']==3 and len(d[0]['fichiers'])==2,f'après relance, Glucose sait encore : {d}')

titre('3. Le fichier déplacé')
loin=os.path.join(T,'ailleurs'); os.makedirs(loin)
c=os.path.join(loin,'eau-deplace.glucose'); shutil.move(a,c)
g2.outil('glucose_domaines'); time.sleep(0.6)
r=g2.attendre_lecture()
d=[x for x in r['domaines'] if x['nom']=='Eau']
print('  après déplacement :',d)
ok(d and d[0]['noeuds']==1,'déplacé loin : il sort de la vue, le fichier reste entier')
g2.geste(ouvrir=c); time.sleep(0.8)
g2.outil('glucose_domaines'); time.sleep(0.5)
r=g2.attendre_lecture()
d=[x for x in r['domaines'] if x['nom']=='Eau']
print('  rouvert :',d)
ok(d and d[0]['noeuds']==3,'rouvert d\'où il est : ses domaines étaient en lui, rien perdu')
b_fichier=g2.outil('glucose_lire')[1]
ok(any(n['id']=='h2o' and n.get('domaines') for n in b_fichier['noeuds']),'ses nœuds portent encore le domaine')
g2.fin()

titre('4. Deux Python, créés sans se connaître')
g3=Glucose('trois')
serpent_dir=os.path.join(T,'projets','zoo'); os.makedirs(serpent_dir)
serpent=os.path.join(serpent_dir,'serpent.glucose')
g3.geste(enregistrer=serpent)
g3.outil('glucose_poser_texte',id='s',texte='Le python est un grand serpent constricteur.',x=0,y=0)
print(' ',g3.outil('glucose_poser_domaine',nouveau='python',ids=['s']))
g3.fin()
g4=Glucose('quatre')
code=os.path.join(serpent_dir,'code.glucose')
g4.geste(enregistrer=os.path.join(T,'projets','code.glucose'))
g4.outil('glucose_poser_texte',id='c',texte='Python est un langage de programmation.',x=0,y=0)
print(' ',g4.outil('glucose_poser_domaine',nouveau='Python',ids=['c']))
# Le fichier du zoo arrive près de celui-ci : deux créations, un nom.
shutil.copy(serpent,os.path.join(T,'projets','serpent-recu.glucose'))
g4.outil('glucose_domaines'); time.sleep(0.6)
r=g4.attendre_lecture()
print('  table :',json.dumps([(x['nom'],x.get('a_preciser',False),x['noeuds']) for x in r['domaines']],ensure_ascii=False))
ok(sum(1 for x in r['domaines'] if x.get('a_preciser'))==2,'les deux sont marqués à préciser : rien réuni en silence')
e,q=g4.outil('glucose_poser_domaine',domaine='python',ids=['c'],poids=0.5)
print('  question :',q)
ok(e=='REFUS' and 'serpent' in q and 'langage' in q and 'meme_sujet' in q,'poser est refusé tant qu\'on n\'a pas répondu, avec les deux sens')
e,r=g4.outil('glucose_poser_domaine',domaine='python',renommer='Python code')
ok(e=='OK',f'l\'IA précise le mot : {r}')
r=g4.attendre_lecture()
print('  table :',json.dumps([(x['nom'],x.get('a_preciser',False),x['noeuds']) for x in r['domaines']],ensure_ascii=False))
ok(not any(x.get('a_preciser') for x in r['domaines']) and {x['nom'] for x in r['domaines']}=={'python','Python code'},'« Python code » et « python » : deux sujets, plus de question')
g4.fin()

titre('5. Ce qu\'il faut lire pour trouver la même chose')
g5=Glucose('cinq')
grand=os.path.join(T,'projets','grand.glucose')
g5.geste(enregistrer=grand)
import random
random.seed(7)
mots='lumière onde photon matière énergie champ noyau atome charge masse vitesse temps espace courbure trou noir étoile galaxie orbite vide'.split()
for i in range(300):
    t=' '.join(' '.join(random.choice(mots) for _ in range(10))+'.' for _ in range(6))
    g5.outil('glucose_poser_texte',id=f'n{i}',texte=f'Note {i}. {t}',x=(i%20)*700,y=(i//20)*500)
sujets={'Optique':range(0,12),'Gravité':range(12,30)}
for nom,plage in sujets.items():
    print(' ',g5.outil('glucose_poser_domaine',nouveau=nom,ids=[f'n{i}' for i in plage])[0])
g5.outil('glucose_lire',niveau='complet'); complet=g5.dernier
g5.outil('glucose_lire'); sommaire=g5.dernier
g5.outil('glucose_domaines'); table=g5.dernier
g5.outil('glucose_domaines',domaine='optique'); nd=g5.dernier
print(f'  tout lire en entier              : {complet:>8} caractères')
print(f'  lire le sommaire de chaque nœud  : {sommaire:>8} caractères')
print(f'  la table des domaines            : {table:>8} caractères')
print(f'  les 12 nœuds d\'un domaine        : {nd:>8} caractères')
print(f'  table + domaine contre sommaire  : {(table+nd)*100/sommaire:.1f} %')
g5.fin()
print('\nFAUTES :',fautes if fautes else 'aucune')
sys.exit(1 if fautes else 0)
