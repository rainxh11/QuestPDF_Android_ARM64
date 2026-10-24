"""Compare real C#/Rust Companion clients against local Postman stand-ins."""
import argparse,base64,datetime,json,os,socket,subprocess,time,urllib.request,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('--output');a=p.parse_args();os.chdir(ROOT)
out=Path(a.output)if a.output else Path('output')/('companion-verification-'+datetime.datetime.now().strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir(parents=True,exist_ok=False)
def request(base,path,body=None):
    data=None if body is None else json.dumps(body).encode()
    return json.load(urllib.request.urlopen(urllib.request.Request(base+path,data=data,headers={'Content-Type':'application/json'}),timeout=15))
def port():
    with socket.socket()as s:s.bind(('127.0.0.1',0));return s.getsockname()[1]
processes=[];handles=[]
def server(name,command,url):
    handle=(out/(name+'.ndjson')).open('w',encoding='utf-8');handles.append(handle)
    flags=subprocess.CREATE_NO_WINDOW|subprocess.CREATE_NEW_PROCESS_GROUP if os.name=='nt'else 0
    process=subprocess.Popen(command,stdout=handle,stderr=subprocess.STDOUT,creationflags=flags);processes.append(process)
    deadline=time.monotonic()+30
    while time.monotonic()<deadline:
        if process.poll()is not None:raise RuntimeError((out/(name+'.ndjson')).read_text())
        if '"listening"'in(out/(name+'.ndjson')).read_text(encoding='utf-8'):
            assert request(url,'/ping')=={};return
        time.sleep(.1)
    raise RuntimeError('Postman server did not start: '+name)
def run(name,command,expected=0):
    result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,encoding='utf-8',errors='replace')
    (out/(name+'.log')).write_text(result.stdout,encoding='utf-8')
    assert (result.returncode==0)==(expected==0),(name,result.returncode,result.stdout[-2000:])
    return result
try:
    direct_port,sim_port=port(),port();direct=f'http://127.0.0.1:{direct_port}';simulation=f'http://127.0.0.1:{sim_port}/companion'
    server('mock',['postman','mock','run','postman/mocks/companion/config.yaml','--port',str(direct_port),'--output','ndjson','--no-history'],direct)
    server('simulation',['postman','simulation','run','postman/simulations/companion.sim.yaml','--port',str(sim_port),'--output','ndjson','--no-history'],simulation)
    run('build-rust',['cargo','build','--locked','-p','questpdf','--example','companion_preview'])
    results={}
    for side,base in [('dotnet',direct),('rust',simulation)]:
        request(base,'/_fixture/reset',{})
        command=['dotnet','run','--project','verification/companion-oracle/CompanionOracle.csproj','-p:WarningLevel=0','--',str(direct_port)]if side=='dotnet'else['cargo','run','--locked','-p','questpdf','--example','companion_preview','--',base]
        run('client-'+side,command)
        result=request(base,'/_fixture/status');results[side]=result
        (out/(side+'.json')).write_text(json.dumps(result,indent=2),encoding='utf-8')
        assert len(result['updates'])==1 and len(result['images'])==1 and not result['errors']
        assert result['updates'][0]['pages']==[dict(width=240,height=180)]*2
        assert result['notifications'],(side,'heartbeat was not observed')
        for notification in result['notifications']:
            uuid.UUID(notification['clientId']);assert notification['license']=='community'
    def frames(result):return{(p['pageIndex'],p['zoomLevel']):base64.b64decode(p['imageData'])for batch in result['images']for p in batch['pages']}
    reference,actual=frames(results['dotnet']),frames(results['rust'])
    assert reference.keys()==actual.keys()=={(0,0),(0,1),(1,0)}
    for key in reference:
        assert reference[key]==actual[key],('JPEG byte difference',key)
        (out/f'page-{key[0]}-zoom-{key[1]}.jpg').write_bytes(actual[key])
    def text_locations(node):
        values=[]
        if node.get('elementType')=='TextBlock'and node.get('searchableContent')in['Companion API v3','Second page']:
            values.append((node['searchableContent'],node['pageLocations']))
        for child in node['children']:values+=text_locations(child)
        return values
    assert text_locations(results['dotnet']['updates'][0]['hierarchy'])==text_locations(results['rust']['updates'][0]['hierarchy'])
    command=['cargo','run','--locked','-p','questpdf','--example','companion_preview','--',simulation]
    request(simulation,'/_fixture/reset',{'supportedVersions':[2]})
    incompatible=run('incompatible-version',command,1);assert 'does not support API version 3'in incompatible.stdout
    assert not request(simulation,'/_fixture/status')['updates']
    request(simulation,'/_fixture/reset',{})
    run('layout-error',command+['invalid']);error=request(simulation,'/_fixture/status');assert len(error['errors'])==1 and not error['updates']
    assert error['errors'][0]['exception']['message']and error['errors'][0]['exception']['type']
    request(simulation,'/_fixture/reset',{})
    run('refresh',command+['refresh']);refresh=request(simulation,'/_fixture/status');assert [x['isDocumentHotReloaded']for x in refresh['updates']]==[False,True]
    assert frames(refresh)==reference
    request(simulation,'/_fixture/reset',{})
    run('run-cancel',command+['run']);assert request(simulation,'/_fixture/status')['notifications']
    # Exercise the actual RIS CLI's --preview route without opening a GUI.
    request(direct,'/_fixture/reset',{})
    handle=(out/'receipt-preview.log').open('w',encoding='utf-8');handles.append(handle)
    flags=subprocess.CREATE_NO_WINDOW if os.name=='nt'else 0
    cli=subprocess.Popen(['cargo','run','--locked','-p','receipt-preview','--','--output',str(out/'cli'),'--report','RECEIPT','--PREVIEW','--companion-port',str(direct_port)],env={**os.environ,'QUESTPDF_TEST_NOW':'2026-10-24T12:00:00'},stdout=handle,stderr=subprocess.STDOUT,creationflags=flags);processes.append(cli)
    deadline=time.monotonic()+45
    while time.monotonic()<deadline:
        assert cli.poll()is None,'ReceiptPreview exited before serving a frame'
        status=request(direct,'/_fixture/status')
        if status['images']:break
        time.sleep(.1)
    else:raise AssertionError('ReceiptPreview did not serve its preview')
    assert len(status['updates'][0]['pages'])==1 and not status['errors']
    assert set(frames(status))=={(0,0),(0,1)}
    (out/'receipt-preview.json').write_text(json.dumps(status,indent=2),encoding='utf-8')
    if os.name=='nt':subprocess.run(['taskkill','/PID',str(processes[0].pid),'/T','/F'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    else:processes[0].terminate()
    assert cli.wait(timeout=15)==0,'ReceiptPreview did not finish when Companion closed'
    for name in ['mock','simulation']:
        log=(out/(name+'.ndjson')).read_text(encoding='utf-8');assert '"request"'in log,'No Postman request events observed'
    print('Companion v3 PASS: independent C#/Rust clients, exact JPEG bytes at two zoom levels, bounds, heartbeats, invalid indices, incompatible API, layout errors, refresh, cancellation and actual ReceiptPreview --preview lifecycle.')
    print(out)
finally:
    for process in processes:
        if process.poll()is None:
            if os.name=='nt':subprocess.run(['taskkill','/PID',str(process.pid),'/T','/F'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
            else:process.terminate()
            try:process.wait(timeout=10)
            except subprocess.TimeoutExpired:process.kill();process.wait()
    for handle in handles:handle.close()
