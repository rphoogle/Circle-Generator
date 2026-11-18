from reportlab.lib.pagesizes import A4
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, Image
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib import colors
from reportlab.lib.units import mm
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
F='/usr/share/fonts/truetype/dejavu/'
pdfmetrics.registerFont(TTFont('DV',F+'DejaVuSans.ttf')); pdfmetrics.registerFont(TTFont('DVB',F+'DejaVuSans-Bold.ttf'))
from reportlab.lib.fonts import addMapping
addMapping('DV',0,0,'DV'); addMapping('DV',1,0,'DVB')
ss=getSampleStyleSheet()
B=ParagraphStyle('b',parent=ss['Normal'],fontName='DV',fontSize=9.5,leading=13.5,spaceAfter=4)
SM=ParagraphStyle('s',parent=B,fontSize=8.5,leading=11.5,spaceAfter=0)
H1=ParagraphStyle('h1',parent=B,fontName='DVB',fontSize=19,leading=24,textColor=colors.HexColor('#1d2433'))
H2=ParagraphStyle('h2',parent=B,fontName='DVB',fontSize=12,leading=16,spaceBefore=8,textColor=colors.HexColor('#2c4a8a'))
BL=ParagraphStyle('bl',parent=B,leftIndent=12,bulletFontName='DV')
P=lambda t,s=B: Paragraph(t,s)
def bul(xs,b='•'): return [Paragraph(x,BL,bulletText=b) for x in xs]
def tbl(rows,w):
    t=Table([[P(c,SM) for c in r] for r in rows],colWidths=w)
    t.setStyle(TableStyle([('GRID',(0,0),(-1,-1),0.4,colors.HexColor('#b9bfcc')),('BACKGROUND',(0,0),(-1,0),colors.HexColor('#e3e8f3')),('VALIGN',(0,0),(-1,-1),'TOP')]))
    return t
s=[P('Circle Generator – user guide',H1),P('Version 0.1 · 18 November 2026'),
 Image('docs/window.png',width=174*mm,height=174*mm*800/1180),
 P('Getting the program',H2)]
s+=bul(['On your GitHub repository, open the <b>Actions</b> tab and click the latest <b>Build Windows .exe</b> run.',
 'Under <b>Artifacts</b> at the bottom, download <b>CircleGenerator-windows</b> (a zip) and take <b>CircleGenerator.exe</b> out of it. Put it anywhere you like.',
 'The first time, Windows SmartScreen will warn you because the program is unsigned: <b>More info</b>, then <b>Run anyway</b>.'])
s+=[P('The controls',H2), tbl([['Control','What it does'],
 ['Shape','Full ●, Half ◓ or Quarter ◔. Halves and quarters need an even diameter; the tool tells you and offers the nearest even sizes.'],
 ['Type','Solid or Hollow. Tick <b>Inverse (hole)</b> for the opposite: a smooth round hole instead of a smooth outside.'],
 ['Fill','Solid: Strips or Square centre. Solid inverse: Strips or Corner blocks. Hollow shapes have no filler.'],
 ['Brick type','Micro (Micro Wedges + micro bricks, any size in micro bricks) or Default / Tile / Smooth (Side Wedges, whole studs, heights in plates). Switching converts your sizes.'],
 ['Diameter / Radius','Linked: change either. The slider goes to 800 studs (the longest single brick); type bigger numbers in the box and long strips are split into equal pieces.'],
 ['Height','Micro: in micro bricks, with bricks and plates shown. Stud types: in plates, with bricks shown.'],
 ['Wall','Solid inverse only: the thickness at the thinnest point between the hole and the edge of the block.'],
 ['Colour / Material','Colour picker or hex code; Plastic, Glass, Translucent, Glow, Metallic or Hologram.'],
 ['File name','Filled in automatically from your settings. Type your own if you like; <b>↺ Use automatic name</b> puts it back.'],
 ['Save to Prefabs','Saves into %LOCALAPPDATA%\\Brickadia\\Saved\\Prefabs. Asks before replacing a file with the same name.'],
 ['Copy to clipboard','Copies the prefab so you can paste it straight into the game with Ctrl+V.']],[34*mm,140*mm]),
 P('Units',H2), tbl([['Unit','Equals'],['1 M','one micro brick: 0.2 stud wide, 0.2 stud tall'],['5 M','1 stud'],['2 M','1 plate'],['6 M (3 plates)','1 brick']],[34*mm,140*mm]),
 P('Every save is checked',H2),
 P('Before anything is saved or copied, the program checks the layout for gaps, overlapping bricks and anything outside the shape. If the check ever fails, nothing is saved and a red message appears at the bottom: please send me the settings.'),
 P('First-run checklist',H2)]
s+=bul(['The window opens and shows the 80 M circle (57 bricks).',
 'Change each control: the preview and brick counts update straight away.',
 'Save to Prefabs: the file appears in the game\'s prefab list.',
 'Copy to clipboard, then Ctrl+V in game: the circle is ready to place.',
 'Save twice with the same name: you are asked before it is replaced.',
 'Switch Micro ↔ Default: sizes convert (80 M ↔ 16 studs, 18 M ↔ 9 plates).',
 'Choose Half with an odd diameter: the warning and the two "Use…" buttons appear.'],'☐')
SimpleDocTemplate('Circle_Generator_User_Guide.pdf',pagesize=A4,leftMargin=18*mm,rightMargin=18*mm,topMargin=14*mm,bottomMargin=14*mm,title='Circle Generator user guide').build(s)
