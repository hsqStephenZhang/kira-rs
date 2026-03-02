#include <stdio.h>
#include <stdarg.h>
#include <sys/time.h>
#include "sylib.h"

/* Input & output functions */
int getint(){int t; scanf("%d",&t); return t; }
int getch(){char c; scanf("%c",&c); return (int)c; }
int getarray(int a[]){
  int n;
  scanf("%d",&n);
  for(int i=0;i<n;i++) scanf("%d",&a[i]);
  return n;
}
void putint(int a){ printf("%d",a);}
void putch(int a){ printf("%c",a); }
void putarray(int n,int a[]){
  printf("%d:",n);
  for(int i=0;i<n;i++) printf(" %d",a[i]);
  printf("\n");
}
void putf(char a[], ...){
  va_list args;
  va_start(args, a);
  vprintf(a, args);
  va_end(args);
}

/* Timing function implementation */
struct timeval _sysy_start,_sysy_end;
#define _SYSY_N 1024
long _sysy_h[_SYSY_N];
long _sysy_m[_SYSY_N];
long _sysy_s[_SYSY_N];
long _sysy_us[_SYSY_N];
int _sysy_idx;

__attribute__((constructor)) void before_main(){
  _sysy_idx = 1;
}

void _sysy_starttime(int lineno){
  _sysy_idx = lineno;
  gettimeofday(&_sysy_start,NULL);
}
void _sysy_stoptime(int lineno){
  gettimeofday(&_sysy_end,NULL);
  _sysy_h[lineno] += _sysy_end.tv_sec - _sysy_start.tv_sec;
  _sysy_us[lineno] += _sysy_end.tv_usec - _sysy_start.tv_usec;
}

// wrappers for simple names if needed
void starttime(){ _sysy_starttime(_sysy_idx); }
void stoptime(){ _sysy_stoptime(_sysy_idx); }
