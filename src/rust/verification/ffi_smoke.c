/* Compile and run against the actual Linux shared library; no .NET runtime. */
#include "ris_document_reports.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main(int argc,char **argv) {
    if(argc!=4)return 2;
    FILE *input=fopen(argv[1],"rb");if(!input)return 3;
    fseek(input,0,SEEK_END);long length=ftell(input);rewind(input);
    uint8_t *json=malloc((size_t)length);if(!json){fclose(input);return 4;}
    if(fread(json,1,(size_t)length,input)!=(size_t)length){fclose(input);free(json);return 5;}
    fclose(input);
    void *pdf=NULL;int32_t pdf_length=0;void *error=NULL;
    int status=doc_reports_generate_pdf(argv[2],json,(int)length,&pdf,&pdf_length,&error);
    free(json);
    if(status!=0){if(error)fprintf(stderr,"%s\n",(char*)error);doc_reports_free_buffer(error);return 6;}
    if(!pdf || pdf_length<5 || memcmp(pdf,"%PDF-",5)!=0){doc_reports_free_buffer(pdf);return 7;}
    FILE *output=fopen(argv[3],"wb");if(!output){doc_reports_free_buffer(pdf);return 8;}
    size_t written=fwrite(pdf,1,(size_t)pdf_length,output);fclose(output);doc_reports_free_buffer(pdf);
    if(written!=(size_t)pdf_length)return 9;
    return 0;
}
