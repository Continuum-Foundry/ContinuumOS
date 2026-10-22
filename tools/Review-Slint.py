"""Use identical independent flat/clip checks and build review sheets for all 30 configurations."""
from pathlib import Path
import json
import runpy
import sys

context=runpy.run_path(str(Path(__file__).with_name('Review-Custom.py')))
run,output,ppm,png=(context[k] for k in ('run','output','ppm','png'))
variants=json.loads(Path('fixtures/m2/ui-v1/variants.json').read_text(encoding='utf-8-sig'))
for variant in variants['variants']:
    sheet=bytearray(1920*900*3)
    for row,step in enumerate(variants['capture_steps']):
        for column,(width,height,scale) in enumerate((w,h,s) for w,h in [(1280,720),(1920,1080)] for s in [100,150,200]):
            name=f"{width}x{height}-{scale}-{variant['id']}-{step}"
            w,h,pixels=ppm(run/(name+'.ppm'))
            for y in range(180):
                sy=min(h-1,int((y+.5)*2*scale/100))
                for x in range(320):
                    sx=min(w-1,int((x+.5)*2*scale/100))
                    source=(sy*w+sx)*3;dest=((row*180+y)*1920+column*320+x)*3
                    sheet[dest:dest+3]=pixels[source:source+3]
    png(output/f"overview-{variant['id']}.png",(1920,900,sheet))
print('Overview sheets sample the same 640x360 logical workspace; selected PNGs preserve full native pixels.')
