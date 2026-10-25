#!/usr/bin/env python3
"""Measure an existing owned desktop process; CPU is percent of one logical core."""
import argparse,json,pathlib,subprocess,time,statistics
p=argparse.ArgumentParser(description=__doc__);p.add_argument('pid',type=int);p.add_argument('output',type=pathlib.Path);p.add_argument('--seconds',type=int,default=30);p.add_argument('--label',required=True);a=p.parse_args()
if not 10<=a.seconds<=300:p.error('Use 10–300 seconds')
def sample():
    fields=subprocess.check_output(['ps','-p',str(a.pid),'-o','time=,rss=,stat='],text=True).split()
    if len(fields)!=3 or 'Z' in fields[2] or int(fields[1])<=0: raise SystemExit('Measurement invalid: process exited or has no resident memory')
    parts=fields[0].split(':');cpu=sum(float(n)*60**i for i,n in enumerate(reversed(parts)))
    return {'at':time.monotonic(),'cpu_seconds':cpu,'rss_mib':int(fields[1])/1024}
rows=[sample()]
for _ in range(a.seconds):
    time.sleep(1);next_sample=sample()
    if next_sample['cpu_seconds']<rows[-1]['cpu_seconds']:raise SystemExit('Measurement invalid: CPU counter moved backwards')
    rows.append(next_sample)
r={'label':a.label,'pid':a.pid,'window_seconds':rows[-1]['at']-rows[0]['at'],'cpu_percent_one_core':(rows[-1]['cpu_seconds']-rows[0]['cpu_seconds'])/(rows[-1]['at']-rows[0]['at'])*100,'rss_observed_max_mib':max(r['rss_mib'] for r in rows),'rss_median_mib':statistics.median(r['rss_mib'] for r in rows),'samples':rows}
a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:v for k,v in r.items() if k!='samples'},indent=2))
