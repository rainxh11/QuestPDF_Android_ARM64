"""Exercise actual exported C functions and allocator ownership via ctypes."""
import argparse,ctypes as c,json
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--library',default='target/debug/ris_document_reports.dll');p.add_argument('--output',default='output/ffi');p.add_argument('--fixtures',default='verification/fixtures');p.add_argument('--printer-reference');a=p.parse_args()
lib=c.CDLL(str(Path(a.library).resolve()))
generate=lib.doc_reports_generate_pdf
generate.argtypes=[c.c_char_p,c.c_char_p,c.c_int,c.POINTER(c.c_void_p),c.POINTER(c.c_int),c.POINTER(c.c_void_p)]
generate.restype=c.c_int
free=lib.doc_reports_free_buffer;free.argtypes=[c.c_void_p]
out=Path(a.output);out.mkdir(parents=True,exist_ok=True)
def call(kind,payload,expected=0):
    data=c.c_void_p(123);length=c.c_int(123);error=c.c_void_p(123)
    status=generate(kind,payload,len(payload)if payload else 0,c.byref(data),c.byref(length),c.byref(error))
    try:
        message=c.string_at(error).decode()if error.value else ''
        assert status==expected,(status,message)
        if status==0:
            assert data.value and length.value>0 and not error.value
            pdf=c.string_at(data,length.value);assert pdf.startswith(b'%PDF-');return pdf
        assert not data.value and length.value==0 and error.value
    finally:
        if data.value:free(data)
        if error.value:free(error)
for kind,name in [('receipt','receipt'),('debt-receipt','debt-receipt'),('large-receipt','large-receipt'),('debt','debt-report'),('debt-out','debt-out-report'),('income','income-report'),('substitute','substitute-report')]:
    payload=(Path(a.fixtures)/(name+'.json')).read_bytes()
    (out/(name+'.pdf')).write_bytes(call(kind.encode(),payload))
for kind,payload in [(None,b'{}'),(b'receipt',None),(b'unknown',b'{}'),(b'receipt',b'{'),(b'receipt',b'null'),(b'receipt',b'{"PaidAmount":"bad"}')]:call(kind,payload,1)
free(None)
assert generate(b'receipt',b'{}',2,None,None,None)==1
# The source accepts PascalCase and case-insensitive property names, and null lists.
payload=json.loads((Path(a.fixtures)/'receipt.json').read_text())
payload={k.lower():v for k,v in payload.items()}
payload.update(exams=None,products=None,transactions=None)
call(b'receipt',json.dumps(payload).encode())
import pymupdf
empty={**json.loads((Path(a.fixtures)/'receipt.json').read_text()),'Exams':None,'Products':None,'Transactions':None}
reference=pymupdf.open(stream=call(b'receipt',json.dumps(empty).encode()),filetype='pdf')[0].get_text()
actual=pymupdf.open(stream=call(b'receipt',json.dumps(payload).encode()),filetype='pdf')[0].get_text()
assert reference==actual,'case-insensitive fields changed document content'
print('C ABI: 7 generation fixtures, invalid inputs, allocator release, null lists and case-insensitive fields passed')
if a.printer_reference:
    import socket,threading,queue
    printer=lib.doc_reports_print_receipt
    printer.argtypes=[c.c_char_p,c.c_char_p,c.c_int,c.c_char_p,c.c_int,c.POINTER(c.c_void_p)];printer.restype=c.c_int
    for name in ['receipt','debt-receipt','large-receipt']:
        # Listen only on this machine; no printer or external address is contacted.
        with socket.socket()as server:
            server.bind(('127.0.0.1',0));server.listen();server.settimeout(30)
            received=queue.Queue()
            def capture():
                try:
                    with server.accept()[0]as connection:
                        connection.settimeout(30);chunks=[]
                        while chunk:=connection.recv(65536):chunks.append(chunk)
                        received.put(b''.join(chunks))
                except BaseException as e:received.put(e)
            thread=threading.Thread(target=capture,daemon=True);thread.start()
            payload=(Path(a.fixtures)/(name+'.json')).read_bytes();error=c.c_void_p()
            status=printer(name.encode(),payload,len(payload),b'127.0.0.1',server.getsockname()[1],c.byref(error))
            try:assert status==0,(status,c.string_at(error).decode()if error.value else '')
            finally:
                if error.value:free(error)
            thread.join(30);assert not thread.is_alive(),'capture timed out'
            data=received.get_nowait()
            if isinstance(data,BaseException):raise data
            assert data==(Path(a.printer_reference)/(name+'-escpos.bin')).read_bytes(),name+' TCP print bytes differ'
            print(name,'PASS loopback TCP transport and reference bytes')
