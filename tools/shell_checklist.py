# Shell tab (v0.4.0) in-game checklist -> shellpack/Shell_Test_Pack_v0.4.0_Checklist.pdf
import sys
from reportlab.lib.pagesizes import A4
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, Image, PageBreak
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib import colors
from reportlab.lib.units import mm
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
F='/usr/share/fonts/truetype/dejavu/'
pdfmetrics.registerFont(TTFont('DV',F+'DejaVuSans.ttf')); pdfmetrics.registerFont(TTFont('DVB',F+'DejaVuSans-Bold.ttf'))
from reportlab.lib.fonts import addMapping
addMapping('DV',0,0,'DV'); addMapping('DV',1,0,'DVB')
out=sys.argv[1] if len(sys.argv)>1 else 'shellpack'
ss=getSampleStyleSheet()
B=ParagraphStyle('b',parent=ss['Normal'],fontName='DV',fontSize=9.5,leading=13.5,spaceAfter=4)
SM=ParagraphStyle('s',parent=B,fontSize=8.5,leading=11.5,spaceAfter=0)
H1=ParagraphStyle('h1',parent=B,fontName='DVB',fontSize=18,leading=23,textColor=colors.HexColor('#1d2433'))
H2=ParagraphStyle('h2',parent=B,fontName='DVB',fontSize=12,leading=16,spaceBefore=8,textColor=colors.HexColor('#2c4a8a'))
L=ParagraphStyle('l',parent=B,leftIndent=12,bulletFontName='DV')
P=lambda t,s=B: Paragraph(t,s)
rows=[['☐','Test','What to check'],
['','<b>S01</b> Dish, your dome rebuilt: 64 studs radius, 21 studs deep, 24 × 8, 3 M thick, metallic','Same size and shape as your Reflective Dome; 192 panels, 1,393 bricks. Bearings hidden inside the panels and in the centre column below.'],
['','<b>S02</b> Same dish, no bearings','Looks identical to S01 with only 552 bricks (60% fewer). Panels paste frozen.'],
['','<b>S03</b> Full sphere, 40 studs across, 16 × 12, checkerboard','Round all the way; pointed panels meet neatly at both poles; checker pattern lines up.'],
['','<b>S04</b> Half-ball dome, upside-down setting, 24 × 8, red/white rings','Strips meet at the top (column hidden above); rim sits flat at the bottom.'],
['','<b>S05</b> Your balloon outline, 12 × 11, red/white gores','Same outline as your Balloon (narrow at the bottom, open at the top); compare side by side.'],
['','<b>S06</b> Balloon, smoother: 24 × 16','Same outline, smoother curve, 384 panels.'],
['','<b>S07</b> Dish with 1-micro panels, bearings visible on the inside face','Small bumps (bearings) only on the inside of the dish, none on the outside.'],
['','<b>S08</b> Cone (lampshade), equal height steps, 16 × 4','Straight sides; 4 bands of equal height; open at both ends.'],
['','<b>S09</b> Dish, equal angle steps, "smooth inside" seams','Rings tighter where it curves most; inside face looks seamless.'],
['','<b>S10</b> Sphere cap set by rim + depth, column inside, bearings visible','Rim 60 studs across, 12 studs deep; visible bearing column points into the bowl.'],
]
data=[[P(c,SM) for c in r] for r in rows]
t=Table(data,colWidths=[8*mm,74*mm,92*mm],repeatRows=1)
t.setStyle(TableStyle([('GRID',(0,0),(-1,-1),0.4,colors.HexColor('#b9bfcc')),('BACKGROUND',(0,0),(-1,0),colors.HexColor('#e3e8f3')),('VALIGN',(0,0),(-1,-1),'TOP')]))
s=[P('Circle Generator v0.4.0 – Shell test pack',H1),P('In-game checklist · 23 November 2026'),
P('Copy the .brz files into your Prefabs folder (%LOCALAPPDATA%\\Brickadia\\Saved\\Prefabs). Each file is named after what it tests.'),
P('In plain English',H2),
P('Each panel is a flat tapered piece (one micro-brick rectangle plus a micro wedge down each slanted side) on its own entity. With bearings on, it is built the same way as your dome: a column of bearings at the centre turns each strip, and each panel has a bearing on its outer end that holds the next panel at the right tilt. Before saving, the tool replays every bearing using the rule measured from your own dome and balloon, and checks each panel lands within 0.001 units of where it should be.'),
P('For every file',H2)]
for x in ['Pastes without errors, all panels in place (not collapsed or swinging).',
          'The pasting outline matches the shape (no big extra space).',
          'No flickering faces on the outside (overlaps are expected hidden inside with "smooth outside").',
          'Panel edges line up between rings and strips: no visible gaps on the outside.',
          'Matches the preview sheet.']:
    s.append(Paragraph(x,L,bulletText='☐'))
s.append(P('Bearing tests (S01, S10)',H2))
for x in ['Unfreeze one panel: it should stay held at its angle by its bearing.',
          'Change a hinge bearing\'s angle slightly: that panel and the rest of its strip swing together.',
          'Change a centre-column bearing\'s angle: the whole strip turns around the middle.']:
    s.append(Paragraph(x,L,bulletText='☐'))
s+= [P('Each test',H2), t, Spacer(1,6), P('Tell me the test number of anything that looks wrong, ideally with a screenshot.'),
     PageBreak(), Image(out+'/Shell_Test_Pack_v0.4.0_Previews.png',width=95*mm,height=190*mm)]
SimpleDocTemplate(out+'/Shell_Test_Pack_v0.4.0_Checklist.pdf',pagesize=A4,leftMargin=18*mm,rightMargin=18*mm,topMargin=16*mm,bottomMargin=16*mm,title='Shell test pack').build(s)
