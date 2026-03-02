#ifndef _SYLIB_H_
#define _SYLIB_H_

#include <stdarg.h>
#include <stdio.h>

int getint();
int getch();
int getarray(int a[]);
void putint(int a);
void putch(int a);
void putarray(int n, int a[]);
void putf(char a[], ...);
void starttime();
void stoptime();
void _sysy_starttime(int lineno);
void _sysy_stoptime(int lineno);

#endif
