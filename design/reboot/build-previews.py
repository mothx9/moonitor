from PIL import Image,ImageDraw
from pathlib import Path
import hashlib,json,re
p=Path('.local/qa/design-reboot/design');out=Path('docs/design-reboot/previews');out.mkdir(exist_ok=True)
manifest=[]
for f in sorted(p.glob('*.jpg')):
 if not re.match(r'[ABC]-(fleet|host|setup|edge-)',f.name):continue
 w,h=(1280,800) if '-1280-' in f.name else (1440,900)
 scales=re.findall(r'dpr ([\d.]+)',f.with_suffix('.txt').read_text()); dpr=float(scales[0]) if scales else 1.7
 raw=Image.open(f).convert('RGB')
 def pink(x,y):
  r,g,b=raw.getpixel((x,y));return r>190 and 20<g<110 and 65<b<170 and r-g>100
 right=min(x for x in range(800,raw.width) if all(pink(x,y) for y in [500,600,700]))
 bottom=min(y for y in range(900,raw.height) if sum(pink(x,y) for x in range(100,1001,100))>=8)
 assert abs(right/w-(bottom-160)/h)<.01,(f,right,bottom)
 im=raw.crop((0,160,right,bottom)).resize((w,h),Image.Resampling.LANCZOS)
 im.save(out/(f.stem+'.png'),optimize=True)
 manifest.append({'preview':f.name.replace('.jpg','.png'),'css_viewport':[w,h],'reported_dpr':dpr,'crop_scale':right/w,'source_sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'image_sha256':hashlib.sha256((out/(f.stem+'.png')).read_bytes()).hexdigest()})
Path('docs/design-reboot/capture-manifest.json').write_text(json.dumps({'platform':'macOS Safari, CSS fixture viewport; not native Iced','method':'CUA window capture; crop browser chrome y=160px; physical crop from pink viewport border; reported DPR retained separately; resize CSS 1:1','snapshots':manifest},indent=2)+'\n')
lines=['# Preview catalog','', 'Design fixtures only. Generated from a running local browser study; no live telemetry.','', '## Principal views','', '| Direction / size | Fleet / dark | Fleet / light | Host / dark | Host / light | Setup / dark | Setup / light |','|---|---|---|---|---|---|---|']
for d in 'ABC':
 for w in [1280,1440]:
  lines.append(f'| {d} / {w} | '+' | '.join(f'[{v} / {t}](previews/{d}-{v}-{w}-{t}.png)' for v in ['fleet','host','setup'] for t in ['dark','light'])+' |')
lines+=['','## Edge compositions','', 'Each scenario is available interactively in the studio; no production capability is inferred.','', '| State | A / 1280 | A / 1440 | B / 1280 | B / 1440 | C / 1280 | C / 1440 |','|---|---|---|---|---|---|---|']
for st in ['filtered','paused','invalid','unavailable','empty','multi-gpu','save-error','no-gpu']:
 lines.append(f'| {st} | '+' | '.join(f'[{d} / {w}](previews/{d}-edge-{st}-{w}-dark.png)' for d in 'ABC' for w in [1280,1440])+' |')
lines+=['','The studio DOM includes an in-page layout report used to identify overflow.','Capture metadata and hashes: [manifest](capture-manifest.json). 1920/4K, native Linux,','final native font scaling and human approval remain pending. Browser crops','do not qualify native rendering or performance.','']
Path('docs/design-reboot/PREVIEW-CATALOG.md').write_text('\n'.join(lines))
canvas=Image.new('RGB',(1600,1200),'#323640');draw=ImageDraw.Draw(canvas)
for i,st in enumerate(['filtered','paused','invalid','unavailable','empty','multi-gpu','save-error','no-gpu']):
 f=out/f'A-edge-{st}-1440-dark.png';im=Image.open(f);im.thumbnail((780,350));x=(i%2)*800+10;y=(i//2)*300+22;im.thumbnail((760,270));canvas.paste(im,(x,y));draw.text((x,y-15),f'A / {st} / fixture',fill='white')
canvas.save(p/'edges-contact.jpg',quality=95)
print(len(manifest),'screenshots')
