#!/bin/zsh
# End-to-end test of the installer backend: local server + fake WoW folder.
set -e
T=$(mktemp -d); SRV=$T/srv; WOW=$T/Wow; mkdir -p $SRV $WOW/Data $WOW/mods
echo x > $WOW/WoW.exe; printf 'mods/VanillaHelpers.dll\r\n' > $WOW/dlls.txt
printf 'USERS OLD FILE' > $WOW/Data/Patch-F.mpq
head -c 3000000 /dev/urandom > $SRV/HDToggle.dll; head -c 5000000 /dev/urandom > $SRV/Patch-F.mpq; echo lua > $SRV/HDSwitch.lua
python3 - $SRV <<'PY'
import hashlib,json,os,sys
d=sys.argv[1]; f=lambda a,dest,sha=None:{"asset":a,"dest":dest,"sha256":sha or hashlib.sha256(open(f"{d}/{a}","rb").read()).hexdigest(),"size":os.path.getsize(f"{d}/{a}")}
m={"release":"v-test","components":[
 {"id":"hdswitch","name":"HD Switch","version":"9.9","description":"x","dll_line":"mods/HDToggle.dll","files":[f("HDToggle.dll","mods/HDToggle.dll"),f("HDSwitch.lua","Interface/AddOns/HDSwitch/HDSwitch.lua")]},
 {"id":"female","name":"F","version":"1","description":"x","files":[f("Patch-F.mpq","Data/Patch-F.mpq")]},
 {"id":"broken","name":"B","version":"1","description":"x","files":[f("Patch-F.mpq","Data/Patch-Q.mpq","0"*64)]}]}
open(f"{d}/manifest.json","w").write(json.dumps(m))
PY
PORT=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1])')
python3 -m http.server $PORT --bind 127.0.0.1 --directory $SRV >/dev/null 2>&1 & SP=$!
trap "kill $SP; rm -rf $T" EXIT; sleep 0.5
cd ${0:A:h}/../src-tauri
HDI_MANIFEST_BASE=http://127.0.0.1:$PORT HDI_TEST_WOW=$WOW HDI_TEST_NO_GAME=1 PATH="$HOME/.cargo/bin:$PATH" cargo test end_to_end -- --nocapture 2>&1 | grep -E '^test |test result|panicked|error' 
