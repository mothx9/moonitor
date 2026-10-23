#!/usr/bin/env python3
"""Paired local llama.cpp HTTP inference, collector off/on. No prompts or outputs persisted.
Run on the target node, inside a bounded cgroup. Own processes are always stopped.
"""
import argparse, hashlib, json, math, pathlib, random, socket, statistics, subprocess, threading, time, urllib.request

def sha(path):
    h=hashlib.sha256()
    with open(path,'rb') as f:
        while chunk:=f.read(4*1024*1024):h.update(chunk)
    return h.hexdigest()
def terminate(p):
    if p and p.poll() is None:
        p.terminate()
        try:p.wait(10)
        except subprocess.TimeoutExpired:p.kill();p.wait()
def request(url,payload,timeout=90):
    body=json.dumps(payload).encode()
    req=urllib.request.Request(url,body,{'Content-Type':'application/json'})
    started=time.perf_counter();first=None;timings=None
    with urllib.request.urlopen(req,timeout=timeout) as response:
        for line in response:
            if not line.startswith(b'data: '):continue
            data=json.loads(line[6:])
            if data.get('content') and first is None:first=time.perf_counter()-started
            if data.get('stop'):timings=data.get('timings')
    if not timings or first is None:raise RuntimeError('Runtime did not expose completion timing/first token')
    return {'ttft_seconds':first,'elapsed_seconds':time.perf_counter()-started,'timings':timings}
def proc(pid):
    s=pathlib.Path(f'/proc/{pid}/stat').read_text().rsplit(')',1)[1].split()
    rss=next(int(l.split()[1]) for l in pathlib.Path(f'/proc/{pid}/status').read_text().splitlines() if l.startswith('VmRSS:'))
    import os
    return {'at':time.perf_counter(),'cpu_seconds':(int(s[11])+int(s[12]))/os.sysconf('SC_CLK_TCK'),'rss_mib':rss/1024}
def quantile(xs,p):
    ys=sorted(xs);at=(len(ys)-1)*p;lo=math.floor(at);hi=math.ceil(at)
    return ys[lo]+(ys[hi]-ys[lo])*(at-lo)
def bootstrap(xs):
    rng=random.Random(23);medians=[statistics.median(rng.choices(xs,k=len(xs))) for _ in range(10000)]
    return [quantile(medians,.025),quantile(medians,.975)]
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--server',required=True,type=pathlib.Path);parser.add_argument('--model',required=True,type=pathlib.Path)
    parser.add_argument('--agent',required=True,type=pathlib.Path);parser.add_argument('--output',required=True,type=pathlib.Path)
    parser.add_argument('--replicates',type=int,default=5);args=parser.parse_args()
    if args.replicates<5:parser.error('At least five paired replications are required')
    existing=[]
    for entry in pathlib.Path('/proc').iterdir():
        try:
            if entry.name.isdigit() and (entry/'comm').read_text().strip()=='moonitor-agent':existing.append(entry.name)
        except OSError:pass
    if existing:raise RuntimeError('A collector is already running; off/on would be invalid')
    args.output.parent.mkdir(parents=True,exist_ok=True)
    with socket.socket() as sock:sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
    url=f'http://127.0.0.1:{port}'
    results={'schema':1,'fixture_prompt':True,'prompt_output_stored':False,'model_sha256':sha(args.model),'model_parts_sha256':{p.name:sha(p) for p in sorted(args.model.parent.glob(args.model.name.split('-00001-of-')[0]+'-*-of-*.gguf'))},'runtime_sha256':sha(args.server),'agent_sha256':sha(args.agent),'runtime_version':subprocess.check_output([str(args.server),'--version'],stderr=subprocess.STDOUT,text=True).strip(),'replicates':[],'limits':{'threads':2,'context':1024,'tokens':512,'gpu_layers':99},'power_scope':'GpuBoard','cpu_power_watts':None,'limitations':['Shared host; concurrent user workloads are not stopped','TTFT p95 from only five requests per condition is exploratory','GPU power polling adds equal observer overhead to both conditions','No whole-device or per-run energy attribution']}
    server=agent=power=None;power_samples=[]
    def read_power():
        for raw in power.stdout:
            try:
                watts,util=map(float,raw.strip().split(','));power_samples.append((time.perf_counter(),watts,util))
            except ValueError:pass
    try:
        with (args.output.parent/'runtime.log').open('w') as log:
            server=subprocess.Popen([str(args.server),'-m',str(args.model),'--host','127.0.0.1','--port',str(port),'-c','1024','-ngl','99','-t','2','-b','128','-ub','128','--parallel','1'],stdout=log,stderr=log)
            deadline=time.monotonic()+180
            while True:
                if server.poll() is not None:raise RuntimeError('Dedicated runtime exited; inspect runtime.log')
                try:
                    with urllib.request.urlopen(url+'/health',timeout=1) as r:
                        if r.status==200:break
                except Exception:
                    if time.monotonic()>deadline:raise RuntimeError('Runtime startup timed out')
                    time.sleep(.5)
            # Fixed non-private fixture; disable prompt cache for each measured request.
            base=('Explain how a CPU and a GPU perform matrix multiplication. Use precise technical language. ' * 10)
            def payload(pair):return {'prompt':f'Benchmark case {pair}. '+base,'n_predict':512,'temperature':0,'seed':23,'ignore_eos':True,'cache_prompt':False,'stream':True}
            request(url+'/completion',payload(-1))
            try:
                power=subprocess.Popen(['nvidia-smi','--query-gpu=power.draw,utilization.gpu','--format=csv,noheader,nounits','--loop-ms=200'],stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,text=True)
                threading.Thread(target=read_power,daemon=True).start()
            except OSError:results['power_scope']=None
            for pair in range(args.replicates):
                row={'pair':pair,'order':['off','on'] if pair%2==0 else ['on','off']}
                for condition in row['order']:
                    if condition=='on':
                        agent=subprocess.Popen([str(args.agent),'--node-id','benchmark','--interval-ms','1000'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
                        time.sleep(3)
                        if agent.poll() is not None:raise RuntimeError('Collector exited')
                        before=proc(agent.pid)
                    start=time.perf_counter();measurement=request(url+'/completion',payload(pair));end=time.perf_counter()
                    if measurement['timings'].get('cache_n',0)!=0 or measurement['timings']['predicted_n']!=512:raise RuntimeError('Unexpected prompt caching or token count')
                    if agent:
                        after=proc(agent.pid);measurement['collector_cpu_percent_one_core']=(after['cpu_seconds']-before['cpu_seconds'])/(after['at']-before['at'])*100
                        measurement['collector_rss_endpoint_mib']=max(before['rss_mib'],after['rss_mib']);terminate(agent);agent=None
                    watts=[w for t,w,u in power_samples if start<=t<=end]
                    measurement['gpu_board_watts_mean']=statistics.mean(watts) if watts else None
                    measurement['gpu_power_sample_count']=len(watts)
                    row[condition]=measurement
                    time.sleep(1)
                results['replicates'].append(row)
                args.output.write_text(json.dumps(results,indent=2)+'\n')
    finally:
        terminate(agent);terminate(power);terminate(server)
    changes=[(r['on']['timings']['predicted_per_second']/r['off']['timings']['predicted_per_second']-1)*100 for r in results['replicates']]
    results['summary']={'throughput_paired_change_percent_median':statistics.median(changes),'throughput_paired_change_bootstrap_ci95':bootstrap(changes),'ttft_off_p95_seconds':quantile([r['off']['ttft_seconds'] for r in results['replicates']],.95),'ttft_on_p95_seconds':quantile([r['on']['ttft_seconds'] for r in results['replicates']],.95),'throughput_preliminary_target_met':statistics.median(changes)>-2}
    args.output.write_text(json.dumps(results,indent=2)+'\n');print(json.dumps(results['summary'],indent=2))
if __name__=='__main__':main()
