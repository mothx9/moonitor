#!/usr/bin/env python3
"""Measure a release desktop and its SSH agents without starting an AI workload."""
import argparse
import datetime
import json
import pathlib
import shlex
import subprocess
import time
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]
REMOTE = r'''
import json, os, pathlib, time
result = {'at': time.monotonic(), 'hz': os.sysconf('SC_CLK_TCK'), 'processes': {}}
for entry in pathlib.Path('/proc').iterdir():
    if not entry.name.isdigit(): continue
    try:
        if (entry / 'comm').read_text().strip() != 'moonitor-agent': continue
        stat = (entry / 'stat').read_text().rsplit(')', 1)[1].split()
        status = (entry / 'status').read_text().splitlines()
        rss = next(int(s.split()[1]) for s in status if s.startswith('VmRSS:'))
        result['processes'][entry.name] = {'ticks': int(stat[11])+int(stat[12]), 'rss_kib': rss}
    except (OSError, ValueError, StopIteration): pass
print(json.dumps(result))
'''


def cpu_seconds(value):
    parts = value.split(':')
    return sum(float(part) * 60**index for index, part in enumerate(reversed(parts)))


def local_snapshot(pid):
    rows = subprocess.check_output(['ps', '-axo', 'pid=,ppid=,time=,rss='], text=True)
    processes = {}
    for line in rows.splitlines():
        fields = line.split()
        if len(fields) == 4 and (int(fields[0]) == pid or int(fields[1]) == pid):
            processes[fields[0]] = {'seconds': cpu_seconds(fields[2]), 'rss_kib': int(fields[3])}
    return {'at': time.monotonic(), 'processes': processes}


def remote_snapshot(host):
    result = subprocess.run(['ssh', '-T', '-oBatchMode=yes', '-oStrictHostKeyChecking=yes',
                             '-oConnectTimeout=6', host, 'python3 -c ' + shlex.quote(REMOTE)],
                            capture_output=True, text=True, timeout=15, check=True)
    return json.loads(result.stdout)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--config', type=pathlib.Path, default=ROOT / '.local/fleet.toml')
    parser.add_argument('--seconds', type=int, default=30)
    parser.add_argument('--output', type=pathlib.Path, default=ROOT / '.local/qa/performance.json')
    args = parser.parse_args()
    if not 10 <= args.seconds <= 300:
        parser.error('Use a measurement window between 10 and 300 seconds.')
    config = tomllib.loads(args.config.read_text())
    hosts = [node['ssh_alias'] for node in config['nodes'] if node['kind'] == 'ssh']
    data_dir = ROOT / '.local/qa/benchmark-data'
    data_dir.mkdir(parents=True, exist_ok=True)
    with (data_dir / 'desktop.log').open('w') as log:
        app = subprocess.Popen([str(ROOT / 'target/release/moonitor'), '--config', str(args.config),
                                '--data-dir', str(data_dir), '--exit-after', str(args.seconds + 15)],
                               stdout=log, stderr=log)
        time.sleep(5)
        start = local_snapshot(app.pid)
        remote_start = {host: remote_snapshot(host) for host in hosts}
        time.sleep(args.seconds)
        end = local_snapshot(app.pid)
        remote_end = {host: remote_snapshot(host) for host in hosts}
        app.wait(timeout=30)
        if app.returncode != 0:
            raise RuntimeError(f'Desktop exited with status {app.returncode}')
    result = {'at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'ai_workload_impact_tested': False, 'desktop': {}, 'agents': {}}
    duration = end['at'] - start['at']
    for pid, first in start['processes'].items():
        last = end['processes'].get(pid)
        if last:
            result['desktop']['app_with_local_collector' if int(pid) == app.pid else 'ssh_' + pid] = {
                'cpu_percent_one_core': (last['seconds'] - first['seconds']) / duration * 100,
                'rss_endpoint_max_mib': max(first['rss_kib'], last['rss_kib']) / 1024,
                'window_seconds': duration}
    for host in hosts:
        first, last = remote_start[host], remote_end[host]
        if len(first['processes']) != 1 or set(first['processes']) != set(last['processes']):
            raise RuntimeError(f'Agent process set changed or was ambiguous on {host}')
        pid = next(iter(first['processes']))
        a, b = first['processes'][pid], last['processes'][pid]
        duration = last['at'] - first['at']
        result['agents'][host] = {'cpu_percent_one_core': (b['ticks']-a['ticks']) / first['hz'] / duration * 100,
                                 'rss_endpoint_max_mib': max(a['rss_kib'], b['rss_kib']) / 1024,
                                 'window_seconds': duration}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
