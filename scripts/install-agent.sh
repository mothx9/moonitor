#!/bin/sh
# Install only a user-owned executable; never add a service or network listener.
set -eu
if [ "$#" -ne 2 ]; then echo 'Usage: install-agent.sh SSH_ALIAS LOCAL_BINARY' >&2; exit 1; fi
agent_host=$1
agent_binary=$2
case "$agent_host" in ''|-*|*[!a-zA-Z0-9._-]*) echo 'Invalid SSH alias' >&2; exit 1;; esac
agent_hash=$(shasum -a 256 "$agent_binary" | cut -d ' ' -f 1)
ssh -T -o BatchMode=yes -o StrictHostKeyChecking=yes -o ConnectTimeout=6 "$agent_host" "mkdir -p .local/bin .local/lib/moonitor/$agent_hash"
scp -q -o BatchMode=yes -o StrictHostKeyChecking=yes -o ConnectTimeout=6 "$agent_binary" "$agent_host:.local/lib/moonitor/$agent_hash/moonitor-agent"
ssh -T -o BatchMode=yes -o StrictHostKeyChecking=yes -o ConnectTimeout=6 "$agent_host" "chmod 755 .local/lib/moonitor/$agent_hash/moonitor-agent && .local/lib/moonitor/$agent_hash/moonitor-agent --help >/dev/null"
ssh -T -o BatchMode=yes -o StrictHostKeyChecking=yes -o ConnectTimeout=6 "$agent_host" "
  set -eu
  if [ -e .local/bin/moonitor-agent ] || [ -L .local/bin/moonitor-agent ]; then
    old_target=\$(readlink .local/bin/moonitor-agent || true)
    case \"\$old_target\" in ../lib/moonitor/*/moonitor-agent) ;; *) echo 'Existing agent is not managed by moonitor; leaving it untouched.' >&2; exit 1;; esac
  fi
  ln -sfn ../lib/moonitor/$agent_hash/moonitor-agent .local/bin/moonitor-agent
"
printf 'Installed moonitor-agent on %s (%s)\n' "$agent_host" "$agent_hash"
