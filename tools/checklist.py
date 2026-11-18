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
H1=ParagraphStyle('h1',parent=B,fontName='DVB',fontSize=18,leading=23,textColor=colors.HexColor('#1d2433'))
H2=ParagraphStyle('h2',parent=B,fontName='DVB',fontSize=12,leading=16,spaceBefore=8,textColor=colors.HexColor('#2c4a8a'))
P=lambda t,s=B: Paragraph(t,s)
rows=[['☐','Test','What to check'],
['','<b>T01</b> Micro, full, solid, square centre, 80 M × 18 M','Identical to your square-centre prefab (57 bricks). Smooth edge, no gaps.'],
['','<b>T02</b> Micro, full, hollow, 80 M','Identical to your hollow prefab (40 bricks).'],
['','<b>T03</b> Strips, 81 M (odd)','Smooth all round; centre strip lines up; no gaps or overlaps (z-fighting flicker = overlap).'],
['','<b>T04</b> Square centre, 81 M (odd)','As T03. Square sits dead centre.'],
['','<b>T05</b> Half, square centre, 80 M','Flat side is straight; rectangle centred on it.'],
['','<b>T06</b> Quarter, square centre, 80 M','Two straight sides at a right angle; square tucked in the corner.'],
['','<b>T07</b> Hollow inverse, 80 M','Smooth side of every wedge faces the hole; four small 1 M bars at N/E/S/W close the ring.'],
['','<b>T08</b> Solid inverse, corner blocks, 80 M, wall 5 M','Square block 90 M across; hole is smooth; wall is 5 M (1 stud) at its thinnest (middle of each side).'],
['','<b>T09</b> Quarter solid inverse, wall 5 M','Square block with a smooth rounded notch in one corner.'],
['','<b>T10</b> Default (Side Wedge + bricks), 16 studs, 9 plates','Side Wedges face outwards; looks angular (expected at this size); height = 3 bricks.'],
['','<b>T11</b> Tile (Side Wedge Tile + tiles), 48 studs, 3 plates','No studs anywhere on top; edge smooth at this size.'],
['','<b>T12</b> Smooth, half, solid inverse, strips, 24 studs, wall 1 stud','Smooth tiles + Side Wedge Tiles; wall 1 stud at its thinnest; flat side straight.'],
['','<b>T13</b> Micro strips, 520 M (104 studs)','Long strips split into pieces automatically (over the 100-stud limit); no gaps where they join.'],
]
data=[[P(c,SM) for c in r] for r in rows]
t=Table(data,colWidths=[8*mm,70*mm,96*mm],repeatRows=1)
t.setStyle(TableStyle([('GRID',(0,0),(-1,-1),0.4,colors.HexColor('#b9bfcc')),('BACKGROUND',(0,0),(-1,0),colors.HexColor('#e3e8f3')),('VALIGN',(0,0),(-1,-1),'TOP')]))
s=[P('Circle Generator – test pack',H1),P('In-game checklist · 17 November 2026'),
P('Copy the .brz files into your Prefabs folder (%LOCALAPPDATA%\\Brickadia\\Saved\\Prefabs), or open one and copy-paste it. Each file is named after what it tests.'),
P('For every file',H2)]
for x in ['Pastes without errors.','The pasting outline matches the bricks exactly (no extra space round the edge).','No flickering faces (that would mean two bricks overlap).','Seen from above, it matches the preview sheet.']:
    s.append(Paragraph(x,ParagraphStyle('l',parent=B,leftIndent=12,bulletFontName='DV'),bulletText='☐'))
s+= [P('Each test',H2), t, Spacer(1,6), P('Tell me the test number of anything that looks wrong, ideally with a screenshot from above.'),
     Image('testpack/Test_pack_preview.png',width=150*mm,height=150*mm*1970/1680)]
SimpleDocTemplate('testpack/Test_pack_checklist.pdf',pagesize=A4,leftMargin=18*mm,rightMargin=18*mm,topMargin=16*mm,bottomMargin=16*mm,title='Circle Generator test pack').build(s)
