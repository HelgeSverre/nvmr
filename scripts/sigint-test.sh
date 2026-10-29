#!/bin/bash
# Slow server -> start install -> SIGINT mid-download and mid-extraction -> check cleanup and resume.
rm -rf /srv2 /tmp/si; mkdir -p /srv2/dist/v22.50.0 /tmp/si
cd /srv2/dist/v22.50.0
V=v22.50.0; SLUG=node-$V-linux-$(uname -m | sed 's/x86_64/x64/;s/aarch64/arm64/')
mkdir -p /tmp/payload/$SLUG/bin; head -c 150000000 /dev/urandom > /tmp/payload/$SLUG/blob; printf '#!/bin/sh\necho v22.50.0\n' > /tmp/payload/$SLUG/bin/node; chmod +x /tmp/payload/$SLUG/bin/node
XZ_OPT=-0 tar -C /tmp/payload -cJf $SLUG.tar.xz $SLUG; sha256sum $SLUG.tar.xz | sed 's/ \*\?/  /' > SHASUMS256.txt
printf 'version\tdate\tfiles\tnpm\tv8\tuv\tzlib\topenssl\tmodules\tlts\tsecurity\nv22.50.0\t2026-01-01\tx\t-\t-\t-\t-\t-\t-\t-\t-\n' > ../index.tab
# throttling server: sends 1MB chunks with a delay, supports Range
cat > /tmp/slow.py <<'PY'
import http.server, time, os, re
class H(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        p=self.translate_path(self.path)
        if not os.path.isfile(p) or not p.endswith('.xz'): return super().do_GET()
        size=os.path.getsize(p); start=0
        m=re.match(r'bytes=(\d+)-', self.headers.get('Range',''))
        if m: start=int(m.group(1)); self.send_response(206); self.send_header('Content-Range',f'bytes {start}-{size-1}/{size}')
        else: self.send_response(200)
        self.send_header('Content-Length',str(size-start)); self.end_headers()
        with open(p,'rb') as f:
            f.seek(start)
            while True:
                b=f.read(1<<20)
                if not b: break
                try: self.wfile.write(b); self.wfile.flush()
                except Exception: return
                time.sleep(float(os.environ.get('DELAY','0.4')))
    def log_message(self,*a): pass
http.server.ThreadingHTTPServer(('127.0.0.1',8098),H).serve_forever()
PY
up() { for _ in $(seq 50); do curl -sf -o /dev/null http://127.0.0.1:8098/dist/index.tab && return; sleep 0.2; done; }
(cd /srv2 && exec python3 /tmp/slow.py >/dev/null 2>&1 &) ; up
export NVM_DIR=/tmp/si NVM_NODEJS_ORG_MIRROR=http://127.0.0.1:8098/dist NVM_NO_PROGRESS=1
F=/src/target/release/nvmr-fetch
# 1) interrupt mid-download
$F install $NVM_NODEJS_ORG_MIRROR $V $NVM_DIR --dest=$NVM_DIR/versions/node/$V --no-progress >/tmp/si.out 2>&1 &
PID=$!; sleep 3; kill -INT $PID; wait $PID; echo "download interrupted rc=$?"
ls -l $NVM_DIR/.cache/bin/$SLUG/ | awk 'NR>1{print $5, $9}'
# 2) resume to completion (fast now)
DELAY=0 pkill -f slow.py; (cd /srv2 && DELAY=0 exec python3 /tmp/slow.py >/dev/null 2>&1 &); sleep 0.5; up
$F install $NVM_NODEJS_ORG_MIRROR $V $NVM_DIR --dest=$NVM_DIR/versions/node/$V --no-progress 2>&1 | tail -3; echo "resume rc=${PIPESTATUS[0]}"
ls $NVM_DIR/versions/node/$V/bin; echo "cached tarball kept: $(ls $NVM_DIR/.cache/bin/$SLUG | tr '\n' ' ')"
# 3) interrupt during extraction (big xz): remove version, restart with cached tarball, SIGINT shortly after start
rm -rf $NVM_DIR/versions
$F install $NVM_NODEJS_ORG_MIRROR $V $NVM_DIR --dest=$NVM_DIR/versions/node/$V --no-progress >/dev/null 2>&1 &
PID=$!
for _ in $(seq 500); do [ -d $NVM_DIR/.cache/bin/$SLUG/files ] && break; sleep 0.01; done
sleep 0.05; kill -INT $PID; wait $PID; echo "extract interrupted rc=$?"
echo "files dir present after interrupt: $([ -d $NVM_DIR/.cache/bin/$SLUG/files ] && echo yes || echo no); version dir: $([ -d $NVM_DIR/versions/node/$V ] && echo yes || echo no)"
ls $NVM_DIR/.cache/bin/$SLUG | tr '\n' ' '; echo
# 4) Ctrl-C on the whole foreground group during `nvm install`: rc 130, the install lock is released
rm -rf $NVM_DIR/versions $NVM_DIR/.cache; pkill -f slow.py; (cd /srv2 && exec python3 /tmp/slow.py >/dev/null 2>&1 &); sleep 0.5; up
setsid bash -c 'NVMR_EXE=/src/target/release/nvmr; . /src/nvm.sh --no-use; nvm install --no-progress v22.50.0 >/dev/null 2>&1; echo "nvm install rc=$?" > /tmp/si.rc' &
sleep 3; kill -INT -- -$!; sleep 1
cat /tmp/si.rc; echo "lock left behind: $(ls $NVM_DIR/.cache/locks 2>/dev/null | tr '\n' ' ')"
pkill -f slow.py  # a live server would hold the caller's output pipe open
