# Renders the installer pages (HTML -> BMP) and measures button rectangles.
import json, os, sys, base64
from playwright.sync_api import sync_playwright
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
for _p in (os.path.join(HERE,"logo.png"), os.path.join(HERE,"..","..","docs","logo.png")):
    if os.path.exists(_p):
        LOGO = "data:image/png;base64," + base64.b64encode(open(_p,"rb").read()).decode(); break
VER = sys.argv[1] if len(sys.argv) > 1 else "0.1.1"
OUT = sys.argv[2] if len(sys.argv) > 2 else "out"
os.makedirs(OUT, exist_ok=True)

T = {
 "it": dict(step="PASSO {n} DI 3", w_title="Benvenuto in Mousely Hub",
   w_text=f"Controlla mouse, macro e webcam senza software pesanti. Questa procedura installerà Mousely Hub {VER} sul tuo computer.",
   b1=["Nessun account","Nessuna telemetria","Open source, licenza MIT"],
   cancel="Annulla", next="Avanti", back="Indietro", install="Installa", browse="Sfoglia…", fin="Fine",
   d_title="Dove vuoi installarlo?", d_text="Mousely Hub verrà installato in questa cartella.", req="SPAZIO RICHIESTO", avail="SPAZIO DISPONIBILE",
   i_title="Installazione in corso", i_text="Attendi, stiamo preparando Mousely Hub.", steps=["Preparazione","Copia dei file","Collegamenti nel menu Start"],
   s_title=f"Mousely Hub {VER} è già installato", s_text="Scegli cosa vuoi fare.",
   s1=("Aggiungi o reinstalla componenti","Ripara l'installazione mantenendo le impostazioni"), s2=("Disinstalla Mousely Hub","Rimuove l'app dal computer"),
   u_title="Mousely Hub è già installato", u_text="Esiste già un'installazione. Scegli cosa vuoi fare.",
   u1=("Disinstalla prima di installare","Rimuove la versione esistente, poi installa"), u2=("Non disinstallare","Installa sopra la versione esistente"),
   un_title="Disinstalla Mousely Hub", un_text="Mousely Hub verrà rimosso dal tuo computer.", un_from="RIMOZIONE DA", un_ck="Elimina anche i dati dell'app (impostazioni e macro)", un_btn="Disinstalla",
   un2_title="Disinstallazione in corso", un2_text="Attendi, stiamo rimuovendo Mousely Hub.",
   f_title="Tutto pronto", f_text=f"Mousely Hub {VER} è stato installato correttamente.", f_ck="Avvia Mousely Hub"),
 "en": dict(step="STEP {n} OF 3", w_title="Welcome to Mousely Hub",
   w_text=f"Control your mouse, macros and webcam without heavy software. This will install Mousely Hub {VER} on your computer.",
   b1=["No account","No telemetry","Open source, MIT license"],
   cancel="Cancel", next="Next", back="Back", install="Install", browse="Browse…", fin="Finish",
   d_title="Where do you want to install it?", d_text="Mousely Hub will be installed in this folder.", req="SPACE REQUIRED", avail="SPACE AVAILABLE",
   i_title="Installing", i_text="Please wait while Mousely Hub is being set up.", steps=["Preparing","Copying files","Start menu shortcuts"],
   s_title=f"Mousely Hub {VER} is already installed", s_text="Choose what you want to do.",
   s1=("Add or reinstall components","Repairs the installation and keeps your settings"), s2=("Uninstall Mousely Hub","Removes the app from this computer"),
   u_title="Mousely Hub is already installed", u_text="An installation already exists. Choose what you want to do.",
   u1=("Uninstall before installing","Removes the existing version, then installs"), u2=("Do not uninstall","Installs over the existing version"),
   un_title="Uninstall Mousely Hub", un_text="Mousely Hub will be removed from your computer.", un_from="REMOVE FROM", un_ck="Also delete app data (settings and macros)", un_btn="Uninstall",
   un2_title="Uninstalling", un2_text="Please wait while Mousely Hub is being removed.",
   f_title="All set", f_text=f"Mousely Hub {VER} was installed successfully.", f_ck="Run Mousely Hub"),
}

CSS = """
*{box-sizing:border-box;margin:0}body{background:#000;font-family:Inter,'Segoe UI',sans-serif}
.p{width:760px;height:500px;position:relative;overflow:hidden;color:#eaf0ff;background:radial-gradient(600px 400px at 85% 0%,#1a2a6c55,transparent),linear-gradient(160deg,#0b1230,#070b1c)}
.sb{position:absolute;left:0;top:0;bottom:0;width:230px;background:linear-gradient(180deg,#0d1640,#070b1c);border-right:1px solid #1d2a5c}
.sb:after{content:"";position:absolute;left:0;right:0;bottom:0;height:120px;background:radial-gradient(200px 80px at 50% 100%,#3b82f655,transparent)}
.sbi{position:absolute;left:24px;right:24px;top:56px;z-index:2}
.tile{width:84px;height:84px;border-radius:22px;background:linear-gradient(135deg,#3b82f6,#7c5cff);display:grid;place-items:center;margin:0 auto 18px;box-shadow:0 0 40px #3b82f688}
.tile img{width:60px;border-radius:14px}.sb h2{text-align:center;font-size:22px}.sb .v{text-align:center;color:#60a5fa;font-weight:700;font-size:13px;margin:4px 0 30px;letter-spacing:1px}
.sb li{list-style:none;font-size:12.5px;color:#b9c4e6;margin:12px 0;display:flex;gap:9px;align-items:center}.sb li:before{content:"✓";color:#fff;background:#3b82f6;border-radius:50%;width:16px;height:16px;font-size:10px;display:grid;place-items:center;flex:none}
.m{position:absolute;left:262px;right:32px;top:44px}
.step{font-size:11px;letter-spacing:2px;color:#7f8fc0;font-weight:700;margin-bottom:8px}
.bars{display:flex;gap:6px;margin-bottom:26px}.bars i{height:4px;width:46px;border-radius:3px;background:#1b2552}.bars i.on{background:linear-gradient(90deg,#3b82f6,#7c5cff)}
h1{font-size:28px;margin-bottom:10px}p{color:#a9b6dc;font-size:14px;line-height:1.55}
.card{background:#0f1740;border:1px solid #25336b;border-radius:14px;padding:14px 16px}
.btn{position:absolute;bottom:50px;height:39px;padding:0 24px;display:grid;place-items:center;border-radius:10px;font-size:14px;font-weight:600;background:#151f4d;border:1px solid #2b3a78;color:#d6defa}
.btn.pr{background:linear-gradient(90deg,#3b82f6,#7c5cff);border:0;color:#fff;box-shadow:0 4px 20px #3b82f666}
.ft{position:absolute;left:262px;right:32px;bottom:16px;display:flex;justify-content:space-between;font-size:11px;color:#6d7bab}
.row{display:flex;gap:12px;margin-top:16px}.row .card{flex:1;height:78px}.row small{color:#7f8fc0;font-size:11px;letter-spacing:1px}
.val{height:30px;margin-top:6px}
.path{display:flex;gap:10px;margin-top:22px}.path .card{flex:1;height:44px;padding:0}.path .b{width:110px;height:44px;display:grid;place-items:center;border-radius:10px;font-size:14px;font-weight:600;background:#151f4d;border:1px solid #2b3a78;color:#d6defa}
.ck{display:flex;gap:12px;align-items:center;height:40px;font-size:15px;color:#cfd8f6}.ck span{width:22px;height:22px;border-radius:50%;border:2px solid #3b82f6;flex:none}
.prog{height:10px;border-radius:6px;background:#1b2552;margin:26px 0 8px}
.pct{height:20px;margin-bottom:6px}
.rad{display:flex;gap:14px;align-items:center;height:68px;margin-top:12px}.rad span{width:20px;height:20px;border-radius:50%;border:2px solid #4b5b94;flex:none}
.rad.s span{border-color:#3b82f6;background:radial-gradient(#3b82f6 45%,transparent 50%)}
.rad b{display:block;font-size:15px}.rad small{color:#8c9bc8;font-size:12px}.rad.s{border-color:#3b82f6}
.ckb{display:flex;gap:12px;align-items:center;height:56px;font-size:15px;color:#cfd8f6;margin-top:24px}
.ckb span{width:22px;height:22px;border-radius:6px;border:2px solid #4b5b94;flex:none;display:grid;place-items:center;font-size:13px;color:#fff}
.ckb.on span{background:#3b82f6;border-color:#3b82f6}.ckb.on{border-color:#3b82f6}
"""

def side(t, bullets):
    li = "".join(f"<li>{b}</li>" for b in t["b1"]) if bullets else ""
    ul = f"<ul>{li}</ul>" if bullets else ""
    return f'<div class=sb><div class=sbi><div class=tile><img src="{LOGO}"></div><h2>Mousely Hub</h2><div class=v>V{VER}</div>{ul}</div></div>'
def bars(n): return '<div class=bars>'+''.join(f'<i class="{"on" if i<n else ""}"></i>' for i in range(3))+'</div>'
def foot(): return '<div class=ft><span>© 2026 Mousely Hub contributors</span><span>MIT License</span></div>'
def btn(id, text, right, pr=False, w=None, dis=False):
    st = f"right:{right}px;" + (f"width:{w}px;" if w else "") + ("opacity:.45;" if dis else "")
    return f'<div id="{id}" class="btn {"pr" if pr else ""}" style="{st}">{text}</div>'

def pages(L):
    t = T[L]; P = {}
    P["p1"] = f'<div class=p>{side(t,True)}<div class=m><div class=step>{t["step"].format(n=1)}</div>{bars(1)}<h1>{t["w_title"]}</h1><p>{t["w_text"]}</p></div>{btn("b-next","%s"%t["next"],32,True,92)}{btn("b-cancel",t["cancel"],136,False,100)}{foot()}</div>'
    P["p2"] = (f'<div class=p>{side(t,False)}<div class=m><div class=step>{t["step"].format(n=2)}</div>{bars(2)}<h1>{t["d_title"]}</h1><p>{t["d_text"]}</p>'
      f'<div class=path><div id="f-path" class=card></div><div id="b-browse" class=b>{t["browse"]}</div></div>'
      f'<div class=row><div class=card><small>{t["req"]}</small><div id="v-req" class=val></div></div><div class=card><small>{t["avail"]}</small><div id="v-avail" class=val></div></div></div></div>'
      f'{btn("b-install",t["install"],32,True,100)}{btn("b-back",t["back"],144,False,100)}{foot()}</div>')
    ck = "".join(f'<div class=ck><span id="c{i+1}"></span>{s}</div>' for i,s in enumerate(t["steps"]))
    P["p3"] = (f'<div class=p>{side(t,False)}<div class=m><div class=step>{t["step"].format(n=3)}</div>{bars(3)}<h1>{t["i_title"]}</h1><p>{t["i_text"]}</p>'
      f'<div id="track" class=prog></div><div id="pct" class=pct></div>{ck}</div>{btn("b-cancel",t["cancel"],32,False,100,True)}{foot()}</div>')
    for key,(ti,te,o1,o2) in (("p4s",(t["s_title"],t["s_text"],t["s1"],t["s2"])),("p4u",(t["u_title"],t["u_text"],t["u1"],t["u2"]))):
        for st in "ab":
            P[f"{key}-{st}"] = (f'<div class=p>{side(t,False)}<div class=m><h1 style="font-size:23px;white-space:nowrap">{ti}</h1><p>{te}</p>'
              f'<div id="card1" class="card rad {"s" if st=="a" else ""}" style="margin-top:22px"><span></span><div><b>{o1[0]}</b><small>{o1[1]}</small></div></div>'
              f'<div id="card2" class="card rad {"s" if st=="b" else ""}"><span></span><div><b>{o2[0]}</b><small>{o2[1]}</small></div></div></div>'
              f'{btn("b-next",t["next"],32,True,92)}{btn("b-cancel",t["cancel"],136,False,100)}{foot()}</div>')
    for st in "ab":
        P[f"p5-{st}"] = (f'<div class=p>{side(t,False)}<div class=m><h1>{t["f_title"]}</h1><p>{t["f_text"]}</p>'
          f'<div id="ck" class="card ckb {"on" if st=="a" else ""}"><span>{"✓" if st=="a" else ""}</span>{t["f_ck"]}</div></div>{btn("b-fin",t["fin"],32,True,100)}{foot()}</div>')
    for st in "ab":
        P[f"u1-{st}"] = (f'<div class=p>{side(t,False)}<div class=m><h1>{t["un_title"]}</h1><p>{t["un_text"]}</p>'
          f'<div class=row style="margin-top:22px"><div class=card style="height:78px"><small style="color:#7f8fc0;font-size:11px;letter-spacing:1px">{t["un_from"]}</small><div id="v-path" class=val></div></div></div>'
          f'<div id="ck" class="card ckb {"on" if st=="b" else ""}" style="margin-top:14px"><span>{"✓" if st=="b" else ""}</span>{t["un_ck"]}</div></div>'
          f'{btn("b-uninst",t["un_btn"],32,True,120)}{btn("b-cancel",t["cancel"],164,False,100)}{foot()}</div>')
    P["u2"] = (f'<div class=p>{side(t,False)}<div class=m><h1>{t["un2_title"]}</h1><p>{t["un2_text"]}</p>'
      f'<div id="track" class=prog></div></div>{btn("b-cancel",t["cancel"],32,False,100,True)}{foot()}</div>')
    return P

rects = {}
STATEFUL={'p4s':['card1','card2'],'p4u':['card1','card2'],'p5':['ck'],'u1':['ck']}
STATELESS={'p1':['b-cancel','b-next'],'p2':['b-browse','b-back','b-install'],'p4s':['b-cancel','b-next'],'p4u':['b-cancel','b-next'],'p5':['b-fin'],'u1':['b-cancel','b-uninst']}
with sync_playwright() as pw:
    b = pw.chromium.launch(); pg = b.new_page(viewport={'width':800,'height':600})
    for L in ("it","en"):
        for name, html in pages(L).items():
            pg.set_content(f'<style>{CSS}</style>{html}')
            pg.evaluate("document.fonts.ready")
            pg.wait_for_timeout(60)
            pg.locator('.p').screenshot(path=f'{OUT}/_t.png')
            Image.open(f'{OUT}/_t.png').convert('RGB').save(f'{OUT}/{name}-{L}.bmp')
            r = pg.evaluate("""()=>{const o={};const p=document.querySelector('.p').getBoundingClientRect();
              document.querySelectorAll('[id]').forEach(e=>{const r=e.getBoundingClientRect();o[e.id]=[Math.round(r.x-p.x),Math.round(r.y-p.y),Math.round(r.width),Math.round(r.height)]});return o}""")
            rects[name.split('-')[0]+"-"+L] = r
            page=name.split('-')[0]; st=name.split('-')[1] if '-' in name else None
            im=Image.open(f'{OUT}/{name}-{L}.bmp')
            for i in STATEFUL.get(page,[]):
                x,y,w,h=r[i]; im.crop((x,y,x+w,y+h)).save(f'{OUT}/c-{page}-{i}-{st}-{L}.bmp')
            if st in (None,'a'):
                for i in STATELESS.get(page,[]):
                    x,y,w,h=r[i]; im.crop((x,y,x+w,y+h)).save(f'{OUT}/c-{page}-{i}-{L}.bmp')
    b.close()
os.remove(f'{OUT}/_t.png')
# check-mark circle (44x44)
from PIL import ImageDraw
S=4; im=Image.new('RGB',(44*S,44*S),(59,130,246)); d=ImageDraw.Draw(im)
d.line([(12*S,23*S),(19*S,30*S),(32*S,15*S)],fill=(255,255,255),width=4*S,joint='curve')
im.resize((44,44),Image.LANCZOS).save(f'{OUT}/chk.bmp')
# sample colours for the opaque labels (it layout == en layout)
def px(name,L,x,y): return '%02X%02X%02X'%Image.open(f'{OUT}/{name}-{L}.bmp').getpixel((x,y))
info = {"rects":{k:v for k,v in rects.items()}}
info["col"] = {"req":px("p2","it",*[rects["p2-it"]["v-req"][0]+3,rects["p2-it"]["v-req"][1]+3]),
               "avail":px("p2","it",*[rects["p2-it"]["v-avail"][0]+3,rects["p2-it"]["v-avail"][1]+3]),
               "field":px("p2","it",*[rects["p2-it"]["f-path"][0]+8,rects["p2-it"]["f-path"][1]+8]),
               "vpath":px("u1-a","it",*[rects["u1-it"]["v-path"][0]+3,rects["u1-it"]["v-path"][1]+3]),
               "pct":px("p3","it",*[rects["p3-it"]["pct"][0]+3,rects["p3-it"]["pct"][1]+3])}
json.dump(info, open(f'{OUT}/rects.json','w'), indent=1)
print(json.dumps(info["col"]), rects["p1-it"]["b-next"])
