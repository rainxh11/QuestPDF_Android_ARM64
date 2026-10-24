from pathlib import Path
import json
def text(value,size=10,**kw):return dict(kind='text',text=value,size=size,**kw)
def col(items,spacing=0):return dict(kind='column',items=items,spacing=spacing)
def wrap(kind,child,value=None,**kw):
    d=dict(kind=kind,child=child,**kw)
    if value is not None:d['value']=value
    return d
def box(child):return wrap('border',wrap('padding',child,[3,2,3,2]),1)
def cell(child,**kw):return dict(content=box(child),**kw)
def table(columns,cells,header=None):
    d=dict(kind='table',columns=columns,cells=cells)
    if header:d['header']=header
    return d
cases=[]
def case(name,content,size=(240,200),**kw):cases.append(dict(name=name,size=size,content=content,expected_error=name.startswith('invalid-'),**kw))
paragraph='Faithful layout preserves line breaks, spacing, measurement, and page transitions. '
case('text-wrap',text(paragraph*8))
case('column-pagination',col([text(f'{i:03} '+paragraph)for i in range(1,41)],3))
case('blank-spacers',col([text('Before'),wrap('height',dict(kind='empty'),30),wrap('padding',dict(kind='empty'),[0,10,0,7]),text('After')],4))
case('center-multiline',wrap('align_center',wrap('width',text(paragraph*2,size=16),150)))
case('right-multiline',wrap('align_right',text(paragraph*2,size=16)))
case('row-widths',dict(kind='row',spacing=3,items=[dict(type='constant',value=40,content=box(text('Constant'))),dict(type='relative',value=2,content=box(text(paragraph))),dict(type='relative',value=1,content=box(text(paragraph)))]))
case('row-auto',dict(kind='row',items=[dict(type='auto',content=box(text('Automatic'))),dict(type='relative',content=box(text('Relative area')))]))
case('row-rtl',dict(kind='row',items=[dict(type='relative',value=v,content=box(text(label)))for v,label in [(1,'First'),(2,'Second'),(1,'Third')]]),rtl=True)
case('page-breaks',col([text('First page'),dict(kind='page_break'),text('Second page'),dict(kind='page_break'),text('Third page')]))
case('ensure-space',col([wrap('height',dict(kind='empty'),110),wrap('ensure_space',text(paragraph*8),60)]))
case('prevent-page-break',col([wrap('height',dict(kind='empty'),110),wrap('prevent_page_break',text(paragraph*8))]))
case('show-entire',col([wrap('height',dict(kind='empty'),105),wrap('show_entire',text(paragraph*2))]))
case('scale-fit',wrap('height',wrap('width',wrap('scale_to_fit',text(paragraph*3,size=24)),180),80))
case('scale',wrap('scale',text(paragraph*4),.7))
for direction in ['rotate_left','rotate_right']:
    case(direction,wrap('height',wrap('width',wrap(direction,text(paragraph,size=14)),120),160))
case('decoration',dict(kind='decoration',before=box(text('Repeated header')),content=col([text(str(i)+' '+paragraph)for i in range(25)]),after=box(text('Repeated footer'))))
case('layers',dict(kind='layers',items=[dict(primary=False,content=wrap('repeat',wrap('background',dict(kind='empty'),'#eeeeee'))),dict(primary=True,content=col([text(str(i)+' '+paragraph)for i in range(15)])),dict(primary=False,content=wrap('repeat',wrap('align_right',text('Watermark'))))]))
case('table-basic',table([1,3,2],[cell(text(f'Cell {i}: '+('detail '*(i%4))))for i in range(60)],[box(text(t,bold=True))for t in ['Number','Description','Amount']]))
case('table-rowspan',table([1,2,2],[cell(text('Spans two rows'),row_span=2),cell(text('Top wide'),column_span=2),cell(text('Bottom left')),cell(text('Bottom right')),cell(text('New row')),cell(text('Last wide'),column_span=2)]))
case('table-columnspan',table([1,2,2],[cell(text('Wide top'),column_span=3)]+[cell(text(f'Data {i}'))for i in range(12)]))
case('table-explicit',table([1,1,1],[cell(text('Last row, column 2'),row=3,column=2),cell(text('Explicit beginning'),row=1,column=1),cell(text('Automatic following')),cell(text('Explicit row'),row=2),cell(text('Explicit column'),column=3)]))
case('table-partial-row',table([1,2],[cell(text('Short')),cell(text(paragraph*15)),cell(text('Tail')),cell(text('Done'))],[box(text('A')),box(text('B'))]))
case('table-rtl',table([1,2,3],[cell(text(f'#{i}'))for i in range(9)]),rtl=True)
case('line-heights',col([text(paragraph,size=14,line_height=v)for v in [.8,1,1.25,1.5]]))
case('continuous',col([box(text(paragraph*2)),text('End')],5),continuous=True)
case('negative-padding',col([text('Top'),wrap('padding',box(text('Offset content')),[-5,-3,-5,-3]),text('After')],5))
case('invalid-width',wrap('width',text('Cannot fit'),300))
case('invalid-table-span',table([1,1],[cell(text('Invalid'),column_span=3)]))
for balanced in (False,True):
    for rtl in (False,True):
        case(f'multicolumn-{balanced}-{rtl}',dict(kind='multi_column',count=3,spacing=10,balance=balanced,content=col([text(f'{i:02} '+paragraph)for i in range(12)],2),spacer=dict(kind='line',value=1)),rtl=rtl,size=(460,300))
case('constraints-overwrite',wrap('width',wrap('width',text('Latest width wins'),120),60))
case('constraints-mixed',wrap('height',wrap('width',box(text('Combined dimensions')),100),60))
case('text-clamp',text(paragraph*5,clamp=3))
case('paragraph-spacing',text((paragraph+'\n')*6,paragraph_spacing=9))
case('paragraph-indent',text((paragraph+'\n')*6,indent=25))
case('paragraph-both-rtl',text((paragraph+'\n')*6,paragraph_spacing=9,indent=25),rtl=True)
for align in ['','left','right','center','justify','around']:
    for rtl in [False,True]:
        case(f'inlined-{align or "default"}-{rtl}',dict(kind='inlined',horizontal=5,vertical=6,align=align,baseline='middle',items=[wrap('width',wrap('height',box(text(f'Item {i}')),22+(i%4)*8),60+(i%4)*16)for i in range(40)]),rtl=rtl)
for alignment in ['AboveBaseline','BelowBaseline','Top','Bottom','Middle']:
    case('inline-'+alignment,dict(kind='rich_text',items=[text('Text before '),dict(kind='inline',alignment=alignment,content=wrap('width',wrap('height',wrap('background',dict(kind='empty'),'#ffbb33'),25),35)),text(' text after '+paragraph)]))
case('inline-alone',dict(kind='rich_text',items=[dict(kind='inline',content=box(text('One embedded text element')))]))
case('text-hyperlinks',dict(kind='rich_text',items=[text(paragraph),dict(kind='link',text=paragraph*5,url='https://example.com/questpdf'),text(' trailing text')]))
case('container-hyperlink',wrap('hyperlink',box(text(paragraph*7)),url='https://example.com/questpdf'))
section_numbers=dict(kind='rich_text',items=[text('Begin '),dict(kind='page_number',type='begin',name='chapter'),text(' end '),dict(kind='page_number',type='end',name='chapter'),text(' total '),dict(kind='page_number',type='section_total',name='chapter'),text(' page '),dict(kind='page_number',type='current',format='padded')])
case('sections',col([section_numbers,dict(kind='rich_text',items=[dict(kind='section_link',text='Go to chapter',name='chapter')]),wrap('section',col([text(str(i)+' '+paragraph)for i in range(20)],2),name='chapter'),wrap('section_link',text('Back to chapter'),name='chapter'),section_numbers]))
case('lazy-pagination',wrap('lazy',col([wrap('lazy',box(text(f'{i} '+paragraph)))for i in range(35)],3)))
case('lazy-multicolumn',dict(kind='multi_column',count=2,spacing=8,balance=True,content=wrap('lazy',col([text(f'{i} '+paragraph)for i in range(14)],2)),spacer=dict(kind='line',value=1)),size=(460,300))
for align in ['left','center','right']:
    for rtl in [False,True]:
        case(f'grid-{align}-{rtl}',dict(kind='grid',count=6,spacing=5,align=align,items=[dict(span=i%3+1,content=box(text(f'Grid item {i} '+paragraph)))for i in range(16)]),rtl=rtl)
case('dynamic-svg',wrap('height',dict(kind='dynamic_svg',text='<svg xmlns="http://www.w3.org/2000/svg" width="100" height="40"><rect width="100" height="40" fill="#ff9900"/><circle cx="50" cy="20" r="15" fill="#335577"/></svg>'),50))
from PIL import Image
import io,base64
image=Image.new('RGB',(35,40),(255,160,50));stream=io.BytesIO();image.save(stream,format='PNG')
case('dynamic-image',wrap('height',dict(kind='dynamic_image',data=base64.b64encode(stream.getvalue()).decode()),70))
import numpy as np
artifacts=Path(__file__).with_name('artifacts');artifacts.mkdir(exist_ok=True)
image_path=artifacts/'compression-source.png'
pixels=np.random.default_rng(2026).integers(0,256,(400,600,3),dtype=np.uint8)
Image.fromarray(pixels).save(image_path)
for quality in ['Best','VeryHigh','High','Medium','Low','VeryLow']:
    case('image-compression-'+quality,wrap('width',dict(kind='image',path='verification/artifacts/compression-source.png',dpi=72,quality=quality),120))
case('image-original',wrap('width',dict(kind='image',path='verification/artifacts/compression-source.png',dpi=72,original=True),120))
case('image-stretch',wrap('height',wrap('width',dict(kind='image',path='verification/artifacts/compression-source.png',stretch=True),120),100))
case('dynamic-pagination',dict(kind='dynamic',count=40))
case('dynamic-pagination-rtl',dict(kind='dynamic',count=40),rtl=True)
case('dynamic-multicolumn',dict(kind='multi_column',count=2,spacing=8,balance=False,content=dict(kind='dynamic',count=40),spacer=dict(kind='line',value=1)),size=(460,300))
case('z-index-order',dict(kind='layers',items=[dict(primary=True,content=wrap('height',wrap('background',text('Main layer'), '#ee8888'),130)),dict(primary=False,content=wrap('z_index',wrap('padding',wrap('background',text('Above'), '#88dd88'),[20,30,80,50]),5)),dict(primary=False,content=wrap('z_index',wrap('background',text('Below'), '#5555ee'),-3))]))
case('z-index-nested',dict(kind='layers',items=[dict(primary=True,content=wrap('height',wrap('background',text('Base'),'#eeeeee'),130)),dict(primary=False,content=wrap('z_index',col([wrap('background',text('Middle'),'#33aa99'),wrap('z_index',wrap('background',text('Highest'),'#eeaa33'),8)]),4)),dict(primary=False,content=wrap('z_index',wrap('padding',text('Between'),[50,20,0,0]),6))]))
captured=wrap('width',wrap('capture',wrap('height',wrap('background',text('Capture'),'#aabbcc'),25),name='anchor'),120)
case('capture-positions',col([text('Before'),captured,wrap('scale',captured,.8),wrap('rotate_right',captured),dict(kind='position_probe',name='anchor')],4),size=(320,400))
for alignment in ['','inside','middle','outside']:
    for radius in [None,[12,5,20,0]]:
        properties=dict(border=[2,4,6,8],border_alignment=alignment)
        if radius:properties['radius']=radius
        case('box-'+(alignment or 'default')+'-'+('rounded'if radius else'square'),wrap('height',wrap('width',wrap('styled_box',wrap('padding',text('Styled box'),[8,8,8,8]),**properties),160),90))
for angle in [0,45,90,180,270,315]:
    case('gradient-'+str(angle),wrap('height',wrap('width',wrap('styled_box',text('Gradient'),border=[4,4,4,4],radius=[10,10,10,10],gradient=angle,border_gradient=angle),160),90))
case('box-shadow',wrap('height',wrap('width',wrap('styled_box',text('Shadow'),border=[1,1,1,1],radius=[10,10,10,10],shadow=True),160),90))
case('system-font-fallback',text('Arial system font: typography 0123456789',font_family='Arial'),system_fonts=True)
semantic_content=wrap('semantic',col([
    wrap('semantic',text('Accessible document',size=18,bold=True),role='H1'),
    text('Automatically tagged paragraph with readable content.'),
    wrap('semantic',text('Explicit paragraph with words.'),role='P'),
    wrap('semantic',text('Decorative ignored text'),role='Ignore'),
    wrap('semantic',wrap('hyperlink',text('Visit example.com'),url='https://example.com'),role='Link',alt='Open example website'),
    wrap('semantic',wrap('height',wrap('width',dict(kind='svg',text='<svg xmlns="http://www.w3.org/2000/svg" width="80" height="30"><rect width="80" height="30" fill="#44aa88"/></svg>'),80),30),role='Figure',alt='A green rectangle'),
    wrap('semantic',col([wrap('semantic',col([wrap('semantic',text('1.'),role='Lbl'),wrap('semantic',text('First list item'),role='LBody')]),role='LI')]),role='L'),
],5),role='Art')
case('semantic-pdfua',semantic_content,pdfua=True,size=(320,400))
case('semantic-pdfa3a',semantic_content,pdfa='PDFA_3A',size=(320,400))
case('semantic-pagination',wrap('semantic',text(paragraph*25),role='P'),pdfua=True)
case('semantic-dynamic',wrap('semantic',dict(kind='dynamic',count=20),role='Art'),pdfua=True)
case('semantic-lazy',wrap('semantic',wrap('lazy',col([text(str(i)+' '+paragraph)for i in range(12)],3)),role='Art'),pdfua=True)
semantic_table=table([1,2,2],[cell(text(f'Row {i//3}, col {i%3}'),semantic_header=i%3==0)for i in range(60)],[box(text('Key')),box(text('Value A')),box(text('Value B'))])
semantic_table['footer']=[box(text('Footer A')),box(text('Footer B')),box(text('Footer C'))]
case('semantic-table-basic',wrap('semantic',semantic_table,role='Table'),pdfua=True)
complex_table=table([1,2,2],[cell(text('Row group'),row_span=2,semantic_header=True),cell(text('Wide data'),column_span=2),cell(text('Lower A')),cell(text('Lower B'))]+[cell(text(f'Data {i}'),semantic_header=i%3==0)for i in range(21)])
complex_table['header_cells']=[cell(text('Group A/B'),column_span=2),cell(text('Column C'),row_span=2),cell(text('Column A')),cell(text('Column B'))]
case('semantic-table-spans',wrap('semantic',complex_table,role='Table'),pdfua=True)
for conformance in ['PDFA_2A','PDFA_2B','PDFA_2U','PDFA_3B','PDFA_3U']:
    case('archival-'+conformance.lower(),semantic_content,pdfa=conformance,size=(320,400))
case('semantic-pdfa-and-ua',semantic_content,pdfa='PDFA_3A',pdfua=True,size=(320,400))
for vertical in [False,True]:
    for gradient in [False,True]:
        case(f'line-dashed-{vertical}-{gradient}',wrap('height',dict(kind='line',value=3,vertical=vertical,gradient=gradient,dash=[2,3,5,2]),100))
case('placeholder-icon',wrap('height',wrap('width',dict(kind='placeholder'),120),80))
case('placeholder-text',wrap('height',wrap('width',dict(kind='placeholder',text='Placeholder'),120),80))
case('repeat-lazy-header',text(paragraph*16),header=wrap('lazy',box(text('Repeated lazy header'))))
case('semantic-repeated-header',text(paragraph*16),header=wrap('lazy',box(text('Repeated lazy header'))),pdfua=True)
for quality in ['Best','VeryHigh','High','Medium','Low','VeryLow']:
    case('dynamic-compression-'+quality,wrap('height',wrap('width',dict(kind='dynamic_image',data=base64.b64encode(image_path.read_bytes()).decode(),dpi=96,quality=quality),120),80))
case('dynamic-original',wrap('height',dict(kind='dynamic_image',data=base64.b64encode(image_path.read_bytes()).decode(),dpi=144,original=True),70))
for label,minimum,maximum in [('both',(70,60),(240,200)),('width',(70,200),(240,200)),('height',(240,60),(240,200)),('minimum',(200,160),(350,300))]:
    case('flexible-page-'+label,col([box(text('Flexible dimensions')),text('End')],5),min_size=minimum,max_size=maximum)
case('flexible-pagination',text(paragraph*20),min_size=(130,80),max_size=(240,200))
case('flexible-rtl',text('Flexible right to left'),min_size=(200,120),max_size=(350,300),rtl=True)
case('global-direction',dict(kind='row',items=[dict(type='relative',content=box(text(label)))for label in ['First','Second','Third']]),global_rtl=True)
case('invalid-page-range',text('Invalid'),min_size=(300,300),max_size=(200,200))

for x,y in [(-1,1),(1,-1),(-1,-1),(.7,1.3)]:
    case(f'scale-axes-{x}-{y}',wrap('height',wrap('width',dict(kind='scale_xy',x=x,y=y,child=box(text('Scaling and flips'))),150),80))
for angle in [30,-45,180]:
    case('rotate-pivot-'+str(angle),wrap('padding',wrap('rotate',box(text('Centered rotation')),angle),[30,30,30,30]))
for rtl in [False,True]:
    case('shrink-'+str(rtl),wrap('height',wrap('shrink',wrap('background',text('Shrink box'),'#bde')),70),rtl=rtl)
    case('unconstrained-'+str(rtl),col([text('Before'),wrap('unconstrained',box(text('Unconstrained'))),text('After')]),rtl=rtl)
case('debug-area',wrap('height',dict(kind='debug_area',text='Debug label',child=box(text('Inner content'))),80))
case('lazy-cached-pagination',wrap('lazy_cached',text(paragraph*20)))
case('conditional-pagination',dict(kind='decoration',before=wrap('show_predicate',box(text('Odd page heading'))),content=text(paragraph*30),after=dict(kind='empty')))
extended=table([1,1,1],[cell(text('Tall left'),row_span=3),cell(text('Right first')),cell(text('Right next'),row=2,column=2),cell(text('Bottom third'),row=4,column=3)])
case('table-extend-last',{**extended,'extend_last':True})

case('stop-paging-partial',wrap('stop_paging',text(paragraph*30)))
case('stop-paging-wrap',col([text('Before'),wrap('stop_paging',wrap('height',text('Too tall'),400)),text('After')]))
case('rotate-coalesced',wrap('rotate',wrap('rotate',box(text('Coalesced rotation')),-15),45))
case('rotate-layout-coalesced',wrap('rotate_right',wrap('rotate_right',wrap('rotate_left',box(text('Layout rotation'))))))

for edge in ['all','horizontal','vertical','before_transparent','after_transparent']:
    case('border-default-'+edge,wrap('height',wrap('width',dict(kind='border_edges',edge=edge,child=text('Default border')),140),60))
case('component-boundary',dict(kind='component_border'))
case('show-if-true-coalescing',wrap('width',wrap('show_bool',wrap('width',text('Latest width wins'),120),True),60))
case('show-if-false',col([text('Before'),wrap('show_bool',wrap('height',text('Hidden'),400),False),text('After')]))
path=Path(__file__).with_name('engine-cases.json');path.write_text(json.dumps(cases,indent=2),encoding='utf-8');print(f'{len(cases)} engine fixtures')
