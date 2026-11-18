import json,sys
from PIL import Image, ImageDraw, ImageFont
F='/usr/share/fonts/truetype/dejavu/'
fT=ImageFont.truetype(F+'DejaVuSans-Bold.ttf',19); fS=ImageFont.truetype(F+'DejaVuSans.ttf',16); fH=ImageFont.truetype(F+'DejaVuSans-Bold.ttf',30)
BG=(250,249,246); FILL=(222,226,236); WEDGE=(120,150,220); EDGE=(70,72,90)
data=json.load(open(sys.argv[1]))
W,H=420,470; cols=4; rows=(len(data)+cols-1)//cols
out=Image.new('RGB',(W*cols,H*rows+90),BG); d=ImageDraw.Draw(out)
d.text((W*cols/2,40),'Circle Generator – test pack 2 (top-down)',font=fH,fill=(25,25,35),anchor='mm')
for i,t in enumerate(data):
    im=Image.new('RGB',(W,H),BG); g=ImageDraw.Draw(im)
    xs=[p[0] for pc in t['pieces'] for p in pc['p']]; ys=[p[1] for pc in t['pieces'] for p in pc['p']]
    ext=max(max(xs)-min(xs),max(ys)-min(ys)); cx=(max(xs)+min(xs))/2; cy=(max(ys)+min(ys))/2
    sc=(W-60)/ext; ox,oy=W/2,60+(W-60)/2
    lw=0 if len(t['pieces'])>200 else 1
    for pc in t['pieces']:
        q=[(ox+(x-cx)*sc, oy-(y-cy)*sc) for x,y in pc['p']]
        g.polygon(q,fill=WEDGE if pc['w'] else FILL,outline=EDGE if lw else None)
    name=t['file'].split('_',1)
    g.text((W/2,20),name[0],font=fT,fill=(30,30,40),anchor='mm')
    g.text((W/2,42),name[1].replace('_',' '),font=fS,fill=(80,80,90),anchor='mm')
    g.text((W/2,H-18),f"{t['bricks']} bricks ({t['wedges']} wedges)",font=fS,fill=(60,60,70),anchor='mm')
    out.paste(im,((i%cols)*W,90+(i//cols)*H))
out.save(sys.argv[2])
