// Local stand-in transcribed from QuestPDF 2026.9.1 CompanionService/Models.
// _fixture routes control the test; production clients use only the v3 contract.
const http = require('http');
const prefix = 'companion.';
const get = async (key, fallback) => (await pm.state.get(prefix + key)) ?? fallback;
const set = (key, value) => pm.state.set(prefix + key, value);
http.createServer(async (req, res) => {
  const path = new URL(req.url, 'http://localhost').pathname;
  const send = (value, status = 200) => {res.writeHead(status, {'Content-Type':'application/json'});res.end(JSON.stringify(value));};
  if (req.method === 'POST' && path === '/_fixture/reset') {
    for (const [key,value] of Object.entries({versions:req.body?.supportedVersions ?? [3],updates:[],images:[],requests:[],errors:[],notifications:[],calls:[]})) await set(key,value);
    return send({});
  }
  if (req.method === 'GET' && path === '/_fixture/status') {
    const status={};for (const key of ['updates','images','errors','notifications','calls'])status[key]=await get(key,[]);return send(status);
  }
  const calls=await get('calls',[]);calls.push({method:req.method,path});await set('calls',calls);
  if(req.method==='GET' && path==='/ping')return send({});
  if(req.method==='GET' && path==='/version')return send({supportedVersions:await get('versions',[3])});
  if(req.method==='POST' && path==='/v3/notify'){
    const values=await get('notifications',[]);values.push(req.body);await set('notifications',values);return send({});
  }
  if(req.method==='POST' && path==='/v3/documentPreview/update'){
    const values=await get('updates',[]);values.push(req.body);await set('updates',values);
    await set('requests',[{pageIndex:0,zoomLevel:0},{pageIndex:0,zoomLevel:1},{pageIndex:1,zoomLevel:0},{pageIndex:-1,zoomLevel:0},{pageIndex:99,zoomLevel:0}]);return send({});
  }
  if(req.method==='GET' && path==='/v3/documentPreview/getRenderingRequests'){
    const requests=await get('requests',[]);await set('requests',[]);if(!requests.length)await new Promise(resolve=>setTimeout(resolve,100));return send(requests);
  }
  if(req.method==='POST' && path==='/v3/documentPreview/provideRenderedImages'){
    const values=await get('images',[]);values.push(req.body);await set('images',values);return send({});
  }
  if(req.method==='POST' && path==='/v3/genericException/show'){
    const values=await get('errors',[]);values.push(req.body);await set('errors',values);return send({});
  }
  send({error:'Endpoint not modeled by the Companion client contract'},501);
}).listen(process.env.PORT || 49127);
